//! Unified Graph Topology (UGT)
//! 
//! Every node is an agent/wallet. Every edge is a credit/liquidity relationship.
//! This is the substrate upon which all six engines operate.

use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use parking_lot::RwLock;

/// Unique identifier for a node (agent/wallet).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

/// Unique identifier for an edge (credit/liquidity relationship).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub u64);

/// A node in the UGT — represents an agent or wallet with thermodynamic state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub wallet_address: String,
    pub capital: f64,
    pub leverage_ratio: f64,
    pub risk_tolerance: f64,  // "threshold potential" in Neural Panic terms
    pub refractory_period_ns: u64,  // capital lock-up time
    pub last_activity_ns: u64,
    pub is_whale: bool,
    pub position_delta: f64,  // net position
    pub health_score: f64,
}

/// An edge in the UGT — represents a credit or liquidity relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub from: NodeId,
    pub to: NodeId,
    pub credit_limit: f64,
    pub current_exposure: f64,
    pub interest_rate: f64,
    pub collateral_ratio: f64,
    pub stress_factor: f64,
    pub last_update_ns: u64,
}

/// The type of credit/liquidity relationship.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CreditRelation {
    DirectLoan,
    LiquidityProvision,
    CrossMargin,
    InsurancePool,
    OTCAgreement,
}

/// The Unified Graph Topology — the complete market graph.
pub struct UnifiedGraphTopology {
    nodes: HashMap<NodeId, Node>,
    edges: HashMap<EdgeId, Edge>,
    adjacency: HashMap<NodeId, HashSet<EdgeId>>,
    next_node_id: u64,
    next_edge_id: u64,
    total_capital: f64,
    total_leverage: f64,
    last_update_ns: u64,
}

impl UnifiedGraphTopology {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: HashMap::with_capacity(capacity),
            edges: HashMap::with_capacity(capacity * 4),
            adjacency: HashMap::with_capacity(capacity),
            next_node_id: 1,
            next_edge_id: 1,
            total_capital: 0.0,
            total_leverage: 0.0,
            last_update_ns: 0,
        }
    }

    pub fn add_node(&mut self, wallet: String, capital: f64) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;

        let node = Node {
            id,
            wallet_address: wallet,
            capital,
            leverage_ratio: 1.0,
            risk_tolerance: 0.5,
            refractory_period_ns: 1_000_000_000, // 1 second default
            last_activity_ns: 0,
            is_whale: capital > 1_000_000.0,
            position_delta: 0.0,
            health_score: 1.0,
        };

        self.total_capital += capital;
        self.nodes.insert(id, node);
        self.adjacency.insert(id, HashSet::new());
        id
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId, credit_limit: f64, relation: CreditRelation) -> EdgeId {
        let id = EdgeId(self.next_edge_id);
        self.next_edge_id += 1;

        let edge = Edge {
            id,
            from,
            to,
            credit_limit,
            current_exposure: 0.0,
            interest_rate: 0.0,
            collateral_ratio: 1.0,
            stress_factor: 0.0,
            last_update_ns: 0,
        };

        self.edges.insert(id, edge);
        self.adjacency.entry(from).or_default().insert(id);
        self.adjacency.entry(to).or_default().insert(id);
        id
    }

    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    pub fn get_edge(&self, id: EdgeId) -> Option<&Edge> {
        self.edges.get(&id)
    }

    pub fn neighbors(&self, node: NodeId) -> Vec<NodeId> {
        self.adjacency.get(&node)
            .map(|edges| {
                edges.iter()
                    .filter_map(|eid| {
                        self.edges.get(eid).map(|e| {
                            if e.from == node { e.to } else { e.from }
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn whale_nodes(&self) -> Vec<&Node> {
        self.nodes.values().filter(|n| n.is_whale).collect()
    }

    pub fn total_systemic_leverage(&self) -> f64 {
        self.nodes.values().map(|n| n.leverage_ratio * n.capital).sum()
    }

    pub fn cluster_coefficient(&self, node: NodeId) -> f64 {
        let neighbors = self.neighbors(node);
        if neighbors.len() < 2 { return 0.0; }

        let mut triangles = 0usize;
        for i in 0..neighbors.len() {
            for j in (i+1)..neighbors.len() {
                let ni_neighbors: HashSet<_> = self.neighbors(neighbors[i]).into_iter().collect();
                if ni_neighbors.contains(&neighbors[j]) {
                    triangles += 1;
                }
            }
        }

        let n = neighbors.len();
        2.0 * triangles as f64 / (n * (n - 1)) as f64
    }

    pub async fn process_event(&mut self, event: crate::MarketEvent) -> Result<(), crate::OmniversaError> {
        match event {
            crate::MarketEvent::WhaleMovement { wallet, delta, timestamp_ns, .. } => {
                if let Some(node) = self.nodes.values_mut()
                    .find(|n| n.wallet_address == wallet) {
                    node.position_delta += delta;
                    node.last_activity_ns = timestamp_ns;
                    node.health_score = (node.health_score * 0.9 + 0.1 * (1.0 - (node.position_delta / node.capital).abs())).max(0.0).min(1.0);
                }
            }
            crate::MarketEvent::Liquidation { symbol: _, side: _, size, price, timestamp_ns } => {
                // Find most leveraged nodes and reduce their health
                let mut affected: Vec<_> = self.nodes.values_mut()
                    .filter(|n| n.leverage_ratio > 5.0)
                    .collect();

                affected.sort_by(|a, b| b.leverage_ratio.partial_cmp(&a.leverage_ratio).unwrap());

                for node in affected.iter_mut().take((size / price).ceil() as usize) {
                    node.health_score *= 0.95;
                    node.leverage_ratio *= 1.05;
                    node.last_activity_ns = timestamp_ns;
                }
            }
            _ => {}
        }

        self.last_update_ns = std::time::Instant::now().elapsed().as_nanos() as u64;
        Ok(())
    }

    pub fn systemic_stress_vector(&self) -> Vec<f64> {
        self.nodes.values()
            .map(|n| n.leverage_ratio * (1.0 - n.health_score))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyConfig {
    pub max_nodes: usize,
    pub max_edges_per_node: usize,
    pub whale_threshold: f64,
    pub stress_propagation_rate: f64,
}
