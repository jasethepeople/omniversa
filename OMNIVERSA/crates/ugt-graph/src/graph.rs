use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    pub id: u64,
    pub weight: f64,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeData {
    pub id: u64,
    pub from: u64,
    pub to: u64,
    pub weight: f64,
    pub directed: bool,
}

#[derive(Debug, Clone)]
pub struct Graph {
    nodes: HashMap<u64, NodeData>,
    edges: HashMap<u64, EdgeData>,
    adjacency: HashMap<u64, HashSet<u64>>,
    next_node_id: u64,
    next_edge_id: u64,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            adjacency: HashMap::new(),
            next_node_id: 1,
            next_edge_id: 1,
        }
    }

    pub fn add_node(&mut self, weight: f64) -> u64 {
        let id = self.next_node_id;
        self.next_node_id += 1;
        self.nodes.insert(id, NodeData {
            id,
            weight,
            attributes: HashMap::new(),
        });
        self.adjacency.insert(id, HashSet::new());
        id
    }

    pub fn add_edge(&mut self, from: u64, to: u64, weight: f64, directed: bool) -> u64 {
        let id = self.next_edge_id;
        self.next_edge_id += 1;
        self.edges.insert(id, EdgeData {
            id,
            from,
            to,
            weight,
            directed,
        });
        self.adjacency.entry(from).or_default().insert(id);
        if !directed {
            self.adjacency.entry(to).or_default().insert(id);
        }
        id
    }

    pub fn neighbors(&self, node: u64) -> Vec<u64> {
        self.adjacency.get(&node)
            .map(|edge_ids| {
                edge_ids.iter()
                    .filter_map(|eid| self.edges.get(eid))
                    .map(|e| if e.from == node { e.to } else { e.from })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn edge_count(&self) -> usize { self.edges.len() }
}
