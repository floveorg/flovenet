use std::time::{Duration, Instant};

use libp2p::gossipsub;
use libp2p::swarm::SwarmEvent;
use tracing::info;

use test_harness::{Scenario, ScenarioResult, TestCheck, TestNode, TestOrchestrator};

// ── Scenario 1: P2P Mesh Formation ─────────────────────────

pub struct P2pMeshScenario {
    pub node_count: usize,
}

#[async_trait::async_trait]
impl Scenario for P2pMeshScenario {
    fn name(&self) -> &str {
        "p2p_mesh"
    }

    async fn run(&self, orch: &TestOrchestrator) -> ScenarioResult {
        let start = std::time::Instant::now();
        let mut checks = Vec::new();

        info!("Creating {count} P2P nodes...", count = self.node_count);
        for i in 0..self.node_count {
            let node = TestNode::new().await;
            info!("  Node {i}: {} on {}", node.peer_id_str(), node.listen_addr);
            orch.add_node(node).await;
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
        info!("Connecting mesh...");
        orch.connect_all().await;

        // Check all nodes exist
        let count = orch.node_count().await;
        checks.push(TestCheck::new(
            "nodes_created",
            count == self.node_count,
            self.node_count.to_string(),
            count.to_string(),
        ));

        let passed = checks.iter().all(|c| c.passed);
        ScenarioResult {
            name: self.name().into(),
            passed,
            duration_ms: start.elapsed().as_millis() as u64,
            checks,
            error: None,
        }
    }
}

// ── Scenario 2: Gossipsub Propagation ──────────────────────

pub struct GossipPropagationScenario {
    pub topic: String,
    pub message: String,
}

#[async_trait::async_trait]
impl Scenario for GossipPropagationScenario {
    fn name(&self) -> &str {
        "gossip_propagation"
    }

    async fn run(&self, orch: &TestOrchestrator) -> ScenarioResult {
        let start = std::time::Instant::now();
        let mut checks = Vec::new();

        let topic = gossipsub::IdentTopic::new(&self.topic);

        // Create 3 nodes, subscribe to topic
        for _ in 0..3 {
            let mut node = TestNode::new().await;
            node.subscribe(&topic);
            orch.add_node(node).await;
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
        orch.connect_all().await;

        // Publish on first node
        {
            let mut nodes = orch.nodes.lock().await;
            info!("Publishing '{}' on topic '{}'", self.message, self.topic);
            nodes[0].publish(&topic, self.message.as_bytes());
        }

        // Poll ALL swarms concurrently until node 1 receives the message.
        // (Only polling the receiver would prevent the sender from actually sending.)
        use futures::StreamExt;
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut received1 = false;
        loop {
            if Instant::now() > deadline {
                break;
            }
            let mut nodes = orch.nodes.lock().await;
            for node in nodes.iter_mut() {
                tokio::select! {
                    event = node.swarm.next() => {
                        if let Some(SwarmEvent::Behaviour(gossipsub::Event::Message { .. })) = event {
                            received1 = true;
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_millis(10)) => {}
                }
                if received1 {
                    break;
                }
            }
            drop(nodes);
            if received1 {
                break;
            }
        }

        checks.push(TestCheck::new(
            "gossip_node1",
            received1,
            "message received",
            if received1 {
                "received"
            } else {
                "not received"
            },
        ));

        let passed = checks.iter().all(|c| c.passed);
        ScenarioResult {
            name: self.name().into(),
            passed,
            duration_ms: start.elapsed().as_millis() as u64,
            checks,
            error: None,
        }
    }
}

// ── Scenario 3: Sequential Messages ─────────────────────────

pub struct SequentialMessagesScenario;

#[async_trait::async_trait]
impl Scenario for SequentialMessagesScenario {
    fn name(&self) -> &str {
        "sequential_messages"
    }

    async fn run(&self, orch: &TestOrchestrator) -> ScenarioResult {
        let start = std::time::Instant::now();
        let mut checks = Vec::new();

        let topic = gossipsub::IdentTopic::new("test/seq");

        // Create 2 nodes
        for _ in 0..2 {
            let mut node = TestNode::new().await;
            node.subscribe(&topic);
            orch.add_node(node).await;
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
        orch.connect_all().await;

        // Send 3 messages in sequence
        let messages = vec!["msg1", "msg2", "msg3"];
        for msg in &messages {
            let mut nodes = orch.nodes.lock().await;
            nodes[0].publish(&topic, msg.as_bytes());
            drop(nodes);
            tokio::time::sleep(Duration::from_millis(400)).await;
        }

        // Poll ALL swarms concurrently to collect messages.
        // (Only polling the receiver would prevent the sender from sending.)
        use futures::StreamExt;
        let mut received: Vec<String> = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(5);
        'collect: loop {
            if Instant::now() > deadline {
                break;
            }
            if received.len() >= messages.len() {
                break;
            }
            let mut nodes = orch.nodes.lock().await;
            for node in nodes.iter_mut() {
                tokio::select! {
                    event = node.swarm.next() => {
                        if let Some(SwarmEvent::Behaviour(gossipsub::Event::Message { message, .. })) = event {
                            let text = String::from_utf8_lossy(&message.data).to_string();
                            if !received.contains(&text) {
                                received.push(text);
                            }
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_millis(10)) => {}
                }
                if received.len() >= messages.len() {
                    break 'collect;
                }
            }
            drop(nodes);
        }

        for &msg in &messages {
            let found = received.iter().any(|r| r.as_str() == msg);
            checks.push(TestCheck::new(
                &format!("received_{msg}"),
                found,
                msg,
                if found { msg } else { "missing" },
            ));
        }

        let passed = checks.iter().all(|c| c.passed);
        ScenarioResult {
            name: self.name().into(),
            passed,
            duration_ms: start.elapsed().as_millis() as u64,
            checks,
            error: None,
        }
    }
}
