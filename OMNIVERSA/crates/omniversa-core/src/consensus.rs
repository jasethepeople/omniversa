//! Sovereign Consensus — Byzantine fault-tolerant state agreement.
//! 
//! In a distributed deployment, multiple OMNIVERSA nodes must agree
//! on the fragility state before acting on it.

use sha3::{Sha3_256, Digest};
use blake3;
use serde::{Serialize, Deserialize};

/// A consensus vote on the current fragility state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragilityVote {
    pub node_id: String,
    pub fragility_hash: [u8; 32],
    pub timestamp_ns: u64,
    pub signature: Vec<u8>,
}

pub struct SovereignConsensus {
    threshold: f64, // fraction of nodes that must agree
    votes: Vec<FragilityVote>,
}

impl SovereignConsensus {
    pub fn new() -> Self {
        Self {
            threshold: 0.67, // 2/3 majority
            votes: Vec::new(),
        }
    }

    pub fn hash_fragility(&self, snapshot: &crate::FragilitySnapshot) -> [u8; 32] {
        let data = serde_json::to_vec(snapshot).unwrap_or_default();
        let mut hasher = Sha3_256::new();
        hasher.update(&data);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    pub fn propose(&mut self, vote: FragilityVote) -> bool {
        self.votes.push(vote);
        self.check_consensus()
    }

    fn check_consensus(&self) -> bool {
        if self.votes.is_empty() { return false; }

        let mut hash_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for vote in &self.votes {
            let key = hex::encode(vote.fragility_hash);
            *hash_counts.entry(key).or_default() += 1;
        }

        let max_agreement = hash_counts.values().max().copied().unwrap_or(0);
        let total = self.votes.len();

        (max_agreement as f64 / total as f64) >= self.threshold
    }
}
