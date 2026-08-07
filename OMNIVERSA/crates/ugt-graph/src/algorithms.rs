use std::collections::{HashMap, VecDeque};
use crate::graph::Graph;

pub struct PageRank;

impl PageRank {
    pub fn compute(graph: &Graph, damping: f64, iterations: usize) -> HashMap<u64, f64> {
        let n = graph.node_count() as f64;
        let mut ranks: HashMap<u64, f64> = HashMap::new();

        // Initialize
        for id in graph.nodes.keys() {
            ranks.insert(*id, 1.0 / n);
        }

        for _ in 0..iterations {
            let mut new_ranks = HashMap::new();

            for id in graph.nodes.keys() {
                let mut rank = (1.0 - damping) / n;

                for neighbor in graph.neighbors(*id) {
                    let out_degree = graph.neighbors(neighbor).len().max(1) as f64;
                    let neighbor_rank = ranks.get(&neighbor).copied().unwrap_or(0.0);
                    rank += damping * neighbor_rank / out_degree;
                }

                new_ranks.insert(*id, rank);
            }

            ranks = new_ranks;
        }

        ranks
    }
}

pub struct BetweennessCentrality;

impl BetweennessCentrality {
    pub fn compute(graph: &Graph) -> HashMap<u64, f64> {
        let mut centrality: HashMap<u64, f64> = HashMap::new();

        for source in graph.nodes.keys() {
            let mut distances: HashMap<u64, usize> = HashMap::new();
            let mut predecessors: HashMap<u64, Vec<u64>> = HashMap::new();
            let mut queue = VecDeque::new();

            distances.insert(*source, 0);
            queue.push_back(*source);

            while let Some(current) = queue.pop_front() {
                let dist = distances[&current];
                for neighbor in graph.neighbors(current) {
                    if !distances.contains_key(&neighbor) {
                        distances.insert(neighbor, dist + 1);
                        queue.push_back(neighbor);
                        predecessors.entry(neighbor).or_default().push(current);
                    } else if distances[&neighbor] == dist + 1 {
                        predecessors.entry(neighbor).or_default().push(current);
                    }
                }
            }

            // Accumulate centrality
            let mut dependency: HashMap<u64, f64> = HashMap::new();
            for id in graph.nodes.keys() {
                dependency.insert(*id, 0.0);
            }

            let mut nodes: Vec<_> = distances.keys().cloned().collect();
            nodes.sort_by(|a, b| distances.get(b).unwrap_or(&0).cmp(distances.get(a).unwrap_or(&0)));

            for node in nodes {
                if let Some(preds) = predecessors.get(&node) {
                    for pred in preds {
                        let pred_dep = dependency.get(pred).copied().unwrap_or(0.0);
                        let node_dep = dependency.get(&node).copied().unwrap_or(0.0);
                        let pred_paths = if *pred == *source { 1.0 } else { pred_dep + 1.0 };
                        let node_paths = if node == *source { 1.0 } else { node_dep + 1.0 };

                        if pred_paths > 0.0 {
                            let delta = (node_paths / pred_paths) * (1.0 + node_dep);
                            *dependency.entry(*pred).or_insert(0.0) += delta;
                        }
                    }
                }
            }

            for (id, dep) in dependency {
                if id != *source {
                    *centrality.entry(id).or_insert(0.0) += dep;
                }
            }
        }

        centrality
    }
}

pub struct CommunityDetection;

impl CommunityDetection {
    pub fn louvain(graph: &Graph, resolution: f64) -> HashMap<u64, usize> {
        let mut communities: HashMap<u64, usize> = HashMap::new();
        let mut next_community = 0usize;

        for id in graph.nodes.keys() {
            communities.insert(*id, next_community);
            next_community += 1;
        }

        let mut improved = true;
        let mut iterations = 0;

        while improved && iterations < 100 {
            improved = false;
            iterations += 1;

            for id in graph.nodes.keys().cloned().collect::<Vec<_>>() {
                let current_comm = communities[&id];
                let mut best_comm = current_comm;
                let mut best_gain = 0.0;

                let neighbor_comms: HashMap<usize, usize> = graph.neighbors(id)
                    .iter()
                    .map(|n| communities.get(n).copied().unwrap_or(current_comm))
                    .fold(HashMap::new(), |mut acc, c| {
                        *acc.entry(c).or_insert(0) += 1;
                        acc
                    });

                for (comm, count) in neighbor_comms {
                    let gain = count as f64 * resolution;
                    if gain > best_gain && comm != current_comm {
                        best_gain = gain;
                        best_comm = comm;
                    }
                }

                if best_comm != current_comm {
                    communities.insert(id, best_comm);
                    improved = true;
                }
            }
        }

        communities
    }
}
