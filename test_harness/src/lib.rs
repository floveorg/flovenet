use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use libp2p::gossipsub::{self, MessageAuthenticity, MessageId};
use libp2p::swarm::SwarmEvent;
use libp2p::{identity, multiaddr::Protocol, Multiaddr, PeerId, Swarm, Transport};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

// ── Data types ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCheck {
    pub name: String,
    pub passed: bool,
    pub expected: String,
    pub actual: String,
}

impl TestCheck {
    pub fn new(
        name: &str,
        passed: bool,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            passed,
            expected: expected.into(),
            actual: actual.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    pub name: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub checks: Vec<TestCheck>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    pub timestamp: String,
    pub total_scenarios: usize,
    pub passed_scenarios: usize,
    pub failed_scenarios: usize,
    pub total_checks: usize,
    pub passed_checks: usize,
    pub duration_ms: u64,
    pub scenarios: Vec<ScenarioResult>,
}

impl TestReport {
    pub fn from_scenarios(scenarios: Vec<ScenarioResult>, duration_ms: u64) -> Self {
        let total = scenarios.len();
        let passed = scenarios.iter().filter(|s| s.passed).count();
        let total_checks: usize = scenarios.iter().map(|s| s.checks.len()).sum();
        let passed_checks: usize = scenarios
            .iter()
            .flat_map(|s| s.checks.iter())
            .filter(|c| c.passed)
            .count();
        Self {
            timestamp: Utc::now().to_rfc3339(),
            total_scenarios: total,
            passed_scenarios: passed,
            failed_scenarios: total - passed,
            total_checks,
            passed_checks,
            duration_ms,
            scenarios,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    pub fn summary(&self) -> String {
        let status = if self.failed_scenarios == 0 {
            "✅ ALL PASSED"
        } else {
            "❌ SOME FAILED"
        };
        format!(
            "{status}\n\
             ─────────────────────────────────────\n\
             Scenarios: {passed}/{total} passed\n\
             Checks:    {ck_p}/{ck_t} passed\n\
             Duration:  {dur_ms}ms\n\
             Time:      {ts}",
            passed = self.passed_scenarios,
            total = self.total_scenarios,
            ck_p = self.passed_checks,
            ck_t = self.total_checks,
            dur_ms = self.duration_ms,
            ts = self.timestamp,
        )
    }
}

// ── Test node ───────────────────────────────────────────────

pub struct TestNode {
    pub peer_id: PeerId,
    pub swarm: Swarm<gossipsub::Behaviour>,
    pub listen_addr: Multiaddr,
}

impl TestNode {
    pub async fn new() -> Self {
        let key = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(key.public());

        let transport =
            libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::new().nodelay(true))
                .upgrade(libp2p::core::upgrade::Version::V1Lazy)
                .authenticate(libp2p::noise::Config::new(&key).unwrap())
                .multiplex(libp2p::yamux::Config::default())
                .boxed();

        let msg_id_fn = |msg: &gossipsub::Message| {
            let data = &msg.data[..msg.data.len().min(64)];
            MessageId::from(xxh3_64(data).to_le_bytes().to_vec())
        };

        let config = gossipsub::ConfigBuilder::default()
            .heartbeat_initial_delay(Duration::from_millis(200))
            .heartbeat_interval(Duration::from_millis(500))
            .message_id_fn(msg_id_fn)
            .validation_mode(gossipsub::ValidationMode::Anonymous)
            .flood_publish(true)
            .build()
            .unwrap();

        let behaviour = gossipsub::Behaviour::new(MessageAuthenticity::Anonymous, config).unwrap();

        let mut swarm = Swarm::new(
            transport,
            behaviour,
            peer_id,
            libp2p::swarm::Config::with_tokio_executor()
                .with_idle_connection_timeout(Duration::from_secs(30)),
        );

        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
        swarm.listen_on(addr).unwrap();

        // Wait for the listen address to be assigned.
        use futures::StreamExt;
        let listen_addr = loop {
            tokio::select! {
                event = swarm.next() => {
                    if let Some(SwarmEvent::NewListenAddr { address, .. }) = event {
                        break address;
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(500)) => {
                    panic!("Timeout waiting for listen address");
                }
            }
        };

        TestNode {
            peer_id,
            swarm,
            listen_addr,
        }
    }

    pub fn subscribe(&mut self, topic: &gossipsub::IdentTopic) {
        self.swarm.behaviour_mut().subscribe(topic).unwrap();
    }

    pub fn publish(&mut self, topic: &gossipsub::IdentTopic, data: &[u8]) {
        self.swarm
            .behaviour_mut()
            .publish(topic.clone(), data)
            .expect("publish should succeed");
    }

    pub fn peer_id_str(&self) -> String {
        self.peer_id.to_string()
    }
}

fn xxh3_64(data: &[u8]) -> u64 {
    let mut h = 0u64;
    for chunk in data.chunks(8) {
        let mut buf = [0u8; 8];
        for (i, b) in chunk.iter().enumerate() {
            buf[i] = *b;
        }
        h ^= u64::from_le_bytes(buf);
        h = h.wrapping_mul(0x9E3779B185EBCA87);
    }
    h
}

// ── Helper: poll a swarm for N ticks, processing events ─────
/// Poll a swarm repeatedly, calling `on_event` for each event.
/// Returns when `on_event` returns `Some(result)` or ticks expire.
pub async fn poll_swarm<F, R>(
    swarm: &mut Swarm<gossipsub::Behaviour>,
    ticks: u32,
    mut on_event: F,
) -> Option<R>
where
    F: FnMut(SwarmEvent<gossipsub::Event>) -> Option<R>,
{
    use futures::StreamExt;
    for _ in 0..ticks {
        tokio::select! {
            event = swarm.next() => {
                if let Some(event) = event {
                    if let Some(r) = on_event(event) {
                        return Some(r);
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(10)) => {}
        }
    }
    None
}

/// Poll a swarm for a number of ticks, discarding events.
pub async fn drain_swarm(swarm: &mut Swarm<gossipsub::Behaviour>, ticks: u32) {
    poll_swarm(swarm, ticks, |_| Some(())).await;
}

/// Wait until a specific `SwarmEvent` variant is seen, or timeout.
pub async fn wait_for_event<F>(
    swarm: &mut Swarm<gossipsub::Behaviour>,
    mut predicate: F,
    timeout_ms: u64,
) -> bool
where
    F: FnMut(&SwarmEvent<gossipsub::Event>) -> bool,
{
    use futures::StreamExt;
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    while Instant::now() < deadline {
        tokio::select! {
            event = swarm.next() => {
                if let Some(event) = event {
                    if predicate(&event) {
                        return true;
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(5)) => {}
        }
    }
    false
}

// ── Orchestrator ───────────────────────────────────────────

pub struct TestOrchestrator {
    pub nodes: Arc<Mutex<Vec<TestNode>>>,
}

impl Default for TestOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl TestOrchestrator {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn add_node(&self, node: TestNode) {
        self.nodes.lock().await.push(node);
    }

    pub async fn node_count(&self) -> usize {
        self.nodes.lock().await.len()
    }

    /// Connect all nodes into a mesh.
    ///
    /// Each node dials every node with a higher index (one direction), then
    /// polls all swarms in rotation until every node has at least one mesh peer
    /// on the given topic (or the 10s deadline expires).
    pub async fn connect_all(&self) -> Vec<TestCheck> {
        let mut checks = Vec::new();
        let n = {
            let nodes = self.nodes.lock().await;
            nodes.len()
        };
        if n < 2 {
            return checks;
        }

        // Dial one direction: node i → node j for i < j
        {
            let mut nodes = self.nodes.lock().await;
            for i in 0..n {
                for j in (i + 1)..n {
                    let target = nodes[j]
                        .listen_addr
                        .clone()
                        .with(Protocol::P2p(nodes[j].peer_id));
                    if let Err(e) = nodes[i].swarm.dial(target) {
                        tracing::warn!("dial node[{i}] → node[{j}] failed: {e:?}");
                    }
                }
            }
        }

        // Poll all swarms until every node has at least (n-1) connected peers.
        use futures::StreamExt;
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if Instant::now() > deadline {
                break;
            }

            let mut nodes = self.nodes.lock().await;

            // Round-robin: poll each swarm -> drain one event or yield 5ms
            for node in nodes.iter_mut() {
                tokio::select! {
                    event = node.swarm.next() => { drop(event); }
                    _ = tokio::time::sleep(Duration::from_millis(5)) => {}
                }
            }

            // Check every node has all other peers connected.
            let need = n - 1;
            let mut all_connected = true;
            for node in nodes.iter() {
                if node.swarm.behaviour().all_peers().count() < need {
                    all_connected = false;
                    break;
                }
            }
            drop(nodes);

            if all_connected {
                break;
            }
        }

        // Final connectivity check.
        let nodes = self.nodes.lock().await;
        let mut all_connected = true;
        let need = n - 1;
        for (i, node) in nodes.iter().enumerate() {
            let count = node.swarm.behaviour().all_peers().count();
            if count < need {
                all_connected = false;
                checks.push(TestCheck::new(
                    &format!("node_{i}_not_all_connected"),
                    false,
                    format!("≥ {need} peers"),
                    format!("{count} peers"),
                ));
            }
        }
        drop(nodes);

        if all_connected {
            checks.push(TestCheck::new(
                "all_connected",
                true,
                format!("{n} nodes"),
                format!("{n} nodes dialed"),
            ));
        }
        checks
    }
}

// ── Scenario trait ─────────────────────────────────────────

#[async_trait::async_trait]
pub trait Scenario: Send + Sync {
    fn name(&self) -> &str;
    async fn run(&self, orch: &TestOrchestrator) -> ScenarioResult;
}
