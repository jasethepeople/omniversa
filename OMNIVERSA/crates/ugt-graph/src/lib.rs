//! UGT Graph — Unified Graph Topology substrate.
//! 
//! This crate provides the graph data structures that all engines share.

#![warn(missing_docs)]
#![deny(unsafe_code)]

use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

pub mod graph;
pub mod algorithms;

pub use graph::{Graph, Node, Edge, NodeData, EdgeData};
pub use algorithms::{PageRank, BetweennessCentrality, CommunityDetection};
