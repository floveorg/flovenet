use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use reputation_engine::ReputationState;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// A signed trust statement from `signer` trusting `target` with a given weight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEdge {
    pub signer: String,
    pub target: String,
    /// Trust weight: 0.0 (no trust) to 1.0 (absolute trust)
    pub weight: f64,
    /// Ed25519 signature over the canonical payload (see `TrustEdge::payload`).
    pub signature: Vec<u8>,
    pub timestamp: DateTime<Utc>,
    /// Ed25519 public key (raw 32 bytes) of `signer`. Optional for
    /// backward compat with edges produced before Hito 0.2; if empty,
    /// `verify()` returns `Unverified` and the graph accepts the edge
    /// with a warning. New code should always populate this.
    #[serde(default)]
    pub signer_pubkey: Vec<u8>,
}

/// Outcome of [`TrustEdge::verify`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyOutcome {
    /// Pubkey present, signature present and valid for the payload.
    Valid,
    /// Pubkey or signature missing — cannot verify, but caller may
    /// still accept (compat mode).
    Unverified,
    /// Pubkey and signature present but signature does NOT match payload.
    Invalid,
}

impl TrustEdge {
    /// Canonical bytes to sign: `signer | target | weight | unix_ts`. The
    /// format is intentionally simple so any client can reproduce it.
    pub fn payload(&self) -> Vec<u8> {
        format!(
            "{}|{}|{}|{}",
            self.signer,
            self.target,
            self.weight,
            self.timestamp.timestamp_millis()
        )
        .into_bytes()
    }

    /// Create a signed edge from a signing key. Used by tests and by
    /// any node that wants to publish a trust statement.
    pub fn signed(
        signer: String,
        target: String,
        weight: f64,
        timestamp: DateTime<Utc>,
        signing_key: &crypto::SigningKey,
    ) -> Self {
        use crypto::Signer;
        let mut edge = TrustEdge {
            signer,
            target,
            weight,
            signature: Vec::new(),
            timestamp,
            signer_pubkey: signing_key.verifying_key().to_bytes().to_vec(),
        };
        let sig = signing_key.sign(&edge.payload());
        edge.signature = sig.to_bytes().to_vec();
        edge
    }

    /// Verify the signature against `signer_pubkey`.
    pub fn verify(&self) -> VerifyOutcome {
        if self.signer_pubkey.is_empty() || self.signature.is_empty() {
            return VerifyOutcome::Unverified;
        }
        let pk_bytes: [u8; 32] = match self.signer_pubkey.as_slice().try_into() {
            Ok(b) => b,
            Err(_) => return VerifyOutcome::Invalid,
        };
        let pk = match crypto::VerifyingKey::from_bytes(&pk_bytes) {
            Ok(k) => k,
            Err(_) => return VerifyOutcome::Invalid,
        };
        let sig_bytes: [u8; 64] = match self.signature.as_slice().try_into() {
            Ok(b) => b,
            Err(_) => return VerifyOutcome::Invalid,
        };
        let sig = crypto::Signature::from_bytes(&sig_bytes);
        use crypto::Verifier;
        match pk.verify(&self.payload(), &sig) {
            Ok(()) => VerifyOutcome::Valid,
            Err(_) => VerifyOutcome::Invalid,
        }
    }
}

/// CRDT-style trust graph: map of (signer, target) -> TrustEdge.
/// Newer timestamps overwrite older ones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustGraph {
    /// Edges keyed by (signer, target) for efficient lookup
    edges: HashMap<(String, String), TrustEdge>,
}

impl Default for TrustGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl TrustGraph {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    /// Add or update a trust edge.
    ///
    /// Hito 0.2: the edge signature is now verified. Behaviour:
    /// - `Invalid` (pubkey+signature present but mismatch) → **rejected**,
    ///   warning logged.
    /// - `Unverified` (missing pubkey or signature) → accepted but
    ///   warning logged. New code should always sign.
    /// - `Valid` → accepted silently.
    ///
    /// Idempotency: if the existing edge is newer, the incoming is dropped.
    pub fn add_edge(&mut self, edge: TrustEdge) {
        match edge.verify() {
            VerifyOutcome::Invalid => {
                warn!(
                    signer = %edge.signer,
                    target = %edge.target,
                    "TrustEdge rejected: signature does not verify"
                );
                return;
            }
            VerifyOutcome::Unverified => {
                warn!(
                    signer = %edge.signer,
                    target = %edge.target,
                    "TrustEdge accepted without verification (no pubkey/signature)"
                );
            }
            VerifyOutcome::Valid => {}
        }

        let key = (edge.signer.clone(), edge.target.clone());
        if let Some(existing) = self.edges.get(&key) {
            if edge.timestamp <= existing.timestamp {
                return; // stale
            }
        }
        self.edges.insert(key, edge);
    }

    /// Merge another trust graph into this one (CRDT LWW merge).
    pub fn merge(&mut self, other: &TrustGraph) {
        for edge in other.edges.values() {
            self.add_edge(edge.clone());
        }
    }

    /// Get direct trust score from `signer` to `target` (0.0 if no edge).
    pub fn direct_trust(&self, signer: &str, target: &str) -> f64 {
        self.edges
            .get(&(signer.to_string(), target.to_string()))
            .map(|e| e.weight)
            .unwrap_or(0.0)
    }

    /// Compute transitive trust from `trustee` to `target` up to a given depth.
    /// Uses second-order transitivity with weight decay.
    /// Formula: trust(A, C) = trust(A, B) × trust(B, C) for each intermediate B,
    /// aggregated via weighted average.
    pub fn transitive_trust(&self, trustee: &str, target: &str, max_depth: usize) -> f64 {
        if max_depth == 0 || trustee == target {
            return self.direct_trust(trustee, target);
        }

        // BFS with depth limit, tracking visited nodes to prevent cycles
        let mut visited: HashSet<String> = HashSet::new();
        visited.insert(trustee.to_string());

        let mut total_trust = self.direct_trust(trustee, target);
        let mut paths_found = if total_trust > 0.0 { 1 } else { 0 };

        // First-degree intermediaries
        let mut current_hop: Vec<String> = self
            .edges
            .values()
            .filter(|e| e.signer == trustee && e.target != target)
            .map(|e| e.target.clone())
            .collect();

        let mut depth = 1;
        while depth < max_depth && !current_hop.is_empty() {
            let mut next_hop = Vec::new();
            for intermediary in &current_hop {
                if visited.contains(intermediary) {
                    continue;
                }
                visited.insert(intermediary.clone());

                let trust_to_intermediary = self.direct_trust(trustee, intermediary);
                let trust_from_intermediary = self.direct_trust(intermediary, target);

                if trust_from_intermediary > 0.0 {
                    // Decay weight: each hop reduces influence by 50%
                    let decay = 1.0 / (2usize.pow(depth as u32)) as f64;
                    let transitive = trust_to_intermediary * trust_from_intermediary * decay;
                    total_trust += transitive;
                    paths_found += 1;
                }

                // Collect next-hop intermediaries
                for edge in self.edges.values() {
                    if edge.signer == *intermediary
                        && edge.target != target
                        && !visited.contains(&edge.target)
                    {
                        next_hop.push(edge.target.clone());
                    }
                }
            }
            current_hop = next_hop;
            depth += 1;
        }

        if paths_found > 0 {
            total_trust / paths_found as f64
        } else {
            0.0
        }
    }

    /// Compute overall trust score that `trustee` has in `target`,
    /// combining direct and second-order transitive trust.
    pub fn trust_score(&self, trustee: &str, target: &str) -> f64 {
        let direct = self.direct_trust(trustee, target);
        let transitive = self.transitive_trust(trustee, target, 2);
        // Direct trust is primary; transitive is supplementary (30% weight)
        direct.max(transitive * 0.3)
    }

    /// Get the list of peers that trust a given peer (incoming edges).
    pub fn trusted_by(&self, peer_id: &str) -> Vec<&TrustEdge> {
        self.edges
            .values()
            .filter(|e| e.target == peer_id)
            .collect()
    }

    /// Get the list of peers that a given peer trusts (outgoing edges).
    pub fn trusts(&self, peer_id: &str) -> Vec<&TrustEdge> {
        self.edges
            .values()
            .filter(|e| e.signer == peer_id)
            .collect()
    }

    /// Select N validators for a given peer_id, combining trust scores and reputation.
    /// Returns the best candidates. Higher trust_from_peer + higher reputation = better.
    pub fn select_validators(
        &self,
        peer_id: &str,
        reputation: &ReputationState,
        count: usize,
        all_peers: &[String],
    ) -> Vec<String> {
        #[allow(dead_code)]
        struct Candidate {
            peer_id: String,
            trust_score: f64,
            rep_score: f64,
            composite: f64,
        }

        let mut candidates: Vec<Candidate> = all_peers
            .iter()
            .filter(|p| *p != peer_id)
            .map(|p| {
                let trust = self.trust_score(peer_id, p);
                let rep = reputation
                    .get_score(p)
                    .map(|s| s.score / 100.0)
                    .unwrap_or(1.0);
                Candidate {
                    peer_id: p.clone(),
                    trust_score: trust,
                    rep_score: rep,
                    composite: trust * 0.6 + rep * 0.4,
                }
            })
            .collect();

        candidates.sort_by(|a, b| {
            b.composite
                .partial_cmp(&a.composite)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        candidates
            .into_iter()
            .take(count)
            .map(|c| c.peer_id)
            .collect()
    }

    /// Total number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Number of unique signers (peers that have issued trust statements).
    pub fn signer_count(&self) -> usize {
        let mut signers = HashSet::new();
        for key in self.edges.keys() {
            signers.insert(key.0.clone());
        }
        signers.len()
    }

    /// Serialize all edges for gossip propagation.
    pub fn to_edges_vec(&self) -> Vec<TrustEdge> {
        self.edges.values().cloned().collect()
    }

    /// Build graph from a list of edges (e.g., from gossip).
    pub fn from_edges(edges: Vec<TrustEdge>) -> Self {
        let mut graph = Self::new();
        for edge in edges {
            graph.add_edge(edge);
        }
        graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_edge(signer: &str, target: &str, weight: f64) -> TrustEdge {
        // Helper for legacy tests: unsigned edge (Unverified path).
        TrustEdge {
            signer: signer.to_string(),
            target: target.to_string(),
            weight,
            signature: vec![],
            timestamp: Utc::now(),
            signer_pubkey: vec![],
        }
    }

    fn make_signed_edge(
        signer_label: &str,
        target: &str,
        weight: f64,
        key: &crypto::SigningKey,
    ) -> TrustEdge {
        TrustEdge::signed(
            signer_label.to_string(),
            target.to_string(),
            weight,
            Utc::now(),
            key,
        )
    }

    #[test]
    fn test_empty_graph() {
        let graph = TrustGraph::new();
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_direct_trust() {
        let mut graph = TrustGraph::new();
        graph.add_edge(make_edge("alice", "bob", 0.8));
        assert!((graph.direct_trust("alice", "bob") - 0.8).abs() < 0.001);
        assert_eq!(graph.direct_trust("bob", "alice"), 0.0);
    }

    #[test]
    fn test_stale_edge_ignored() {
        let mut graph = TrustGraph::new();
        let old = TrustEdge {
            signer: "alice".into(),
            target: "bob".into(),
            weight: 0.8,
            signature: vec![],
            timestamp: DateTime::from_timestamp(1000, 0).unwrap(),
            signer_pubkey: vec![],
        };
        let new = TrustEdge {
            signer: "alice".into(),
            target: "bob".into(),
            weight: 0.2,
            signature: vec![],
            timestamp: Utc::now(),
            signer_pubkey: vec![],
        };
        graph.add_edge(old);
        graph.add_edge(new);
        // Newer (lower weight but newer timestamp) should win
        assert!((graph.direct_trust("alice", "bob") - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_transitive_trust_first_order() {
        let mut graph = TrustGraph::new();
        graph.add_edge(make_edge("alice", "bob", 0.9));
        graph.add_edge(make_edge("bob", "carol", 0.8));
        // Alice -> Bob = 0.9, Bob -> Carol = 0.8
        // transitive alice -> carol via bob: 0.9 * 0.8 * 0.5 (depth=1 decay) = 0.36
        let transitive = graph.transitive_trust("alice", "carol", 2);
        assert!((transitive - 0.36).abs() < 0.001);
    }

    #[test]
    fn test_crdt_merge() {
        let mut graph_a = TrustGraph::new();
        graph_a.add_edge(make_edge("alice", "bob", 0.8));

        let mut graph_b = TrustGraph::new();
        graph_b.add_edge(make_edge("carol", "dave", 0.9));

        graph_a.merge(&graph_b);
        assert_eq!(graph_a.edge_count(), 2);
        assert!((graph_a.direct_trust("alice", "bob") - 0.8).abs() < 0.001);
        assert!((graph_a.direct_trust("carol", "dave") - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_trusted_by() {
        let mut graph = TrustGraph::new();
        graph.add_edge(make_edge("alice", "bob", 0.7));
        graph.add_edge(make_edge("carol", "bob", 0.6));
        let trusted = graph.trusted_by("bob");
        assert_eq!(trusted.len(), 2);
    }

    #[test]
    fn test_select_validators() {
        let mut graph = TrustGraph::new();
        graph.add_edge(make_edge("alice", "bob", 0.9));
        graph.add_edge(make_edge("alice", "carol", 0.5));

        let mut rep = ReputationState::new();
        rep.apply_events(&[
            reputation_engine::ReputationEvent {
                peer_id: "bob".into(),
                timestamp: Utc::now(),
                kind: reputation_engine::EventKind::Contribution {
                    hours: 50.0,
                    uptime_pct: 99.0,
                },
            },
            reputation_engine::ReputationEvent {
                peer_id: "carol".into(),
                timestamp: Utc::now(),
                kind: reputation_engine::EventKind::Contribution {
                    hours: 10.0,
                    uptime_pct: 50.0,
                },
            },
        ]);
        rep.recompute_all();

        let all = vec!["bob".into(), "carol".into(), "dave".into()];
        let validators = graph.select_validators("alice", &rep, 2, &all);
        // Bob should be first (higher trust + higher rep)
        assert_eq!(validators.len(), 2);
        // Bob has higher trust (0.9) and higher rep, so should be first
        let bob_idx = validators.iter().position(|p| p == "bob").unwrap();
        let carol_idx = validators.iter().position(|p| p == "carol").unwrap();
        assert!(bob_idx < carol_idx);
    }

    #[test]
    fn test_trusts_returns_outgoing() {
        let mut graph = TrustGraph::new();
        graph.add_edge(make_edge("alice", "bob", 0.7));
        graph.add_edge(make_edge("alice", "carol", 0.5));
        let outgoing = graph.trusts("alice");
        assert_eq!(outgoing.len(), 2);
        assert!(outgoing.iter().any(|e| e.target == "bob"));
    }

    #[test]
    fn test_trusts_nonexistent() {
        let graph = TrustGraph::new();
        assert!(graph.trusts("nobody").is_empty());
        assert!(graph.trusted_by("nobody").is_empty());
    }

    #[test]
    fn test_self_trust() {
        let mut graph = TrustGraph::new();
        graph.add_edge(make_edge("alice", "alice", 1.0));
        assert_eq!(graph.edge_count(), 1);
        assert_eq!(graph.direct_trust("alice", "alice"), 1.0);
    }

    #[test]
    fn test_merge_with_empty() {
        let mut graph = TrustGraph::new();
        graph.add_edge(make_edge("alice", "bob", 0.8));
        let empty = TrustGraph::new();
        graph.merge(&empty);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_from_edges_roundtrip() {
        let mut graph = TrustGraph::new();
        graph.add_edge(make_edge("a", "b", 0.5));
        graph.add_edge(make_edge("b", "c", 0.9));
        let edges = graph.to_edges_vec();
        let restored = TrustGraph::from_edges(edges);
        assert_eq!(restored.edge_count(), 2);
        assert_eq!(restored.direct_trust("a", "b"), 0.5);
    }

    #[test]
    fn test_select_validators_empty_all_peers() {
        let graph = TrustGraph::new();
        let rep = ReputationState::new();
        let validators = graph.select_validators("alice", &rep, 5, &[]);
        assert!(validators.is_empty());
    }

    #[test]
    fn test_signer_count() {
        let mut graph = TrustGraph::new();
        graph.add_edge(make_edge("alice", "bob", 0.5));
        graph.add_edge(make_edge("alice", "carol", 0.3));
        graph.add_edge(make_edge("bob", "carol", 0.7));
        assert_eq!(graph.signer_count(), 2);
    }

    #[test]
    fn test_transitive_trust_no_path() {
        let graph = TrustGraph::new();
        let trust = graph.transitive_trust("alice", "carol", 2);
        assert_eq!(trust, 0.0);
    }

    #[test]
    fn test_trust_score_combination() {
        let mut graph = TrustGraph::new();
        graph.add_edge(make_edge("alice", "bob", 0.6));
        graph.add_edge(make_edge("bob", "carol", 0.5));
        let score = graph.trust_score("alice", "carol");
        // Direct = 0.0, transitive = 0.6 * 0.5 * 0.5 = 0.15
        // Score = max(0.0, 0.15 * 0.3) = 0.045
        assert!((score - 0.045).abs() < 0.001);
    }

    // ───────── Hito 0.2: signature verification ─────────

    #[test]
    fn test_signed_edge_roundtrip_valid() {
        let key = crypto::generate_keypair();
        let edge = TrustEdge::signed("alice".into(), "bob".into(), 0.7, Utc::now(), &key);
        assert_eq!(edge.verify(), VerifyOutcome::Valid);
    }

    #[test]
    fn test_unsigned_edge_is_unverified() {
        let edge = make_edge("alice", "bob", 0.5);
        assert_eq!(edge.verify(), VerifyOutcome::Unverified);
    }

    #[test]
    fn test_tampered_weight_is_invalid() {
        let key = crypto::generate_keypair();
        let mut edge = TrustEdge::signed("alice".into(), "bob".into(), 0.7, Utc::now(), &key);
        // attacker modifies weight after signing
        edge.weight = 1.0;
        assert_eq!(edge.verify(), VerifyOutcome::Invalid);
    }

    #[test]
    fn test_wrong_pubkey_is_invalid() {
        let key_alice = crypto::generate_keypair();
        let key_eve = crypto::generate_keypair();
        let mut edge = TrustEdge::signed("alice".into(), "bob".into(), 0.7, Utc::now(), &key_alice);
        // Eve tries to claim she's alice by swapping the embedded pubkey
        edge.signer_pubkey = key_eve.verifying_key().to_bytes().to_vec();
        assert_eq!(edge.verify(), VerifyOutcome::Invalid);
    }

    #[test]
    fn test_add_edge_rejects_invalid_signature() {
        let key = crypto::generate_keypair();
        let mut edge = TrustEdge::signed("alice".into(), "bob".into(), 0.7, Utc::now(), &key);
        edge.weight = 0.1; // tamper
        let mut graph = TrustGraph::new();
        graph.add_edge(edge);
        assert_eq!(graph.edge_count(), 0);
        assert_eq!(graph.direct_trust("alice", "bob"), 0.0);
    }

    #[test]
    fn test_add_edge_accepts_valid_signature() {
        let key = crypto::generate_keypair();
        let edge = make_signed_edge("alice", "bob", 0.8, &key);
        let mut graph = TrustGraph::new();
        graph.add_edge(edge);
        assert_eq!(graph.edge_count(), 1);
        assert!((graph.direct_trust("alice", "bob") - 0.8).abs() < 1e-9);
    }

    #[test]
    fn test_add_edge_accepts_unverified_with_warning() {
        // Backward-compat path: unsigned edges still go in (warn-only).
        let edge = make_edge("alice", "bob", 0.5);
        let mut graph = TrustGraph::new();
        graph.add_edge(edge);
        assert_eq!(graph.edge_count(), 1);
    }
}
