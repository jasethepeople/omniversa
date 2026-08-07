//! Cascade Hawkes Engine — The "Seismology" Layer
//! 
//! Scientific Basis: Self-Exciting Point Processes / LAMMPS (Molecular Dynamics)
//! Analogy: Market crashes are tectonic failures.
//! 
//! Models the recursive nature of stop-losses and liquidations. Treats limit orders
//! as molecular bonds. A "fracture" in one price level increases stress on the next.
//! The Hawkes process calculates the branching ratio: probability that one liquidation
//! triggers >1 subsequent liquidation.

#![warn(missing_docs)]
#![deny(unsafe_code)]

use nalgebra::{DVector, DMatrix};
use ndarray::{Array1, Array2};
use rayon::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use statrs::distribution::{Exp, Continuous};

/// Configuration for the Hawkes engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HawkesConfig {
    pub base_intensity: f64,
    pub self_excitation_alpha: f64,
    pub decay_beta: f64,
    pub max_branching_ratio: f64,
    pub price_levels: usize,
    pub molecular_bond_strength: f64,
}

impl Default for HawkesConfig {
    fn default() -> Self {
        Self {
            base_intensity: 0.1,
            self_excitation_alpha: 0.5,
            decay_beta: 2.0,
            max_branching_ratio: 1.5,
            price_levels: 1000,
            molecular_bond_strength: 1.0,
        }
    }
}

/// A "molecular bond" — represents limit order support at a price level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MolecularBond {
    pub price_level: f64,
    pub volume: f64,
    pub stress: f64,
    pub bond_strength: f64,
    pub neighbors: Vec<usize>,
}

/// The Hawkes engine — self-exciting point process for liquidation cascades.
#[derive(Debug, Clone)]
pub struct HawkesEngine {
    config: HawkesConfig,
    bonds: Vec<MolecularBond>,
    event_history: VecDeque<(f64, f64)>,
    current_intensity: Array1<f64>,
    branching_ratio_estimate: f64,
    total_events: u64,
}

impl HawkesEngine {
    /// Initialize the Hawkes engine.
    pub fn new(config: HawkesConfig) -> Result<Self, String> {
        let bonds = (0..config.price_levels)
            .map(|i| MolecularBond {
                price_level: i as f64 * 0.01,
                volume: 1000.0,
                stress: 0.0,
                bond_strength: config.molecular_bond_strength,
                neighbors: vec![
                    i.saturating_sub(1),
                    (i + 1).min(config.price_levels - 1),
                ],
            })
            .collect();

        Ok(Self {
            config: config.clone(),
            bonds,
            event_history: VecDeque::with_capacity(10000),
            current_intensity: Array1::zeros(config.price_levels),
            branching_ratio_estimate: 0.0,
            total_events: 0,
        })
    }

    /// Evaluate cascade risk given systemic stress.
    pub fn evaluate(&self, stress: &[f64]) -> Result<HawkesMetrics, String> {
        let mean_stress = if stress.is_empty() { 0.0 } else {
            stress.iter().sum::<f64>() / stress.len() as f64
        };

        let branching_ratio = self.compute_branching_ratio(mean_stress);
        let intensity = self.current_intensity.sum();
        let expected_cascades = if branching_ratio < 1.0 {
            intensity / (1.0 - branching_ratio)
        } else {
            f64::INFINITY
        };

        Ok(HawkesMetrics {
            branching_ratio,
            intensity,
            expected_cascades,
        })
    }

    fn compute_branching_ratio(&self, stress: f64) -> f64 {
        let alpha = self.config.self_excitation_alpha * (1.0 + stress * 2.0);
        let beta = self.config.decay_beta;
        (alpha / beta).min(self.config.max_branching_ratio)
    }

    /// Ingest a liquidation event and propagate stress through molecular bonds.
    pub fn ingest_liquidation(&mut self, price: f64, volume: f64, timestamp: f64) {
        self.total_events += 1;

        let idx = self.find_nearest_bond(price);
        if idx >= self.bonds.len() { return; }

        let impact = volume / self.bonds[idx].volume;
        self.bonds[idx].stress = (self.bonds[idx].stress + impact).min(1.0);

        self.propagate_stress(idx, impact * 0.5);
        self.update_intensity(timestamp, impact);

        self.branching_ratio_estimate = self.compute_branching_ratio(
            self.bonds.iter().map(|b| b.stress).sum::<f64>() / self.bonds.len() as f64
        );
    }

    fn find_nearest_bond(&self, price: f64) -> usize {
        self.bonds.iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (a.price_level - price).abs()
                    .partial_cmp(&(b.price_level - price).abs())
                    .unwrap()
            })
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn propagate_stress(&mut self, origin: usize, initial_stress: f64) {
        let mut queue = vec![(origin, initial_stress)];
        let mut visited = std::collections::HashSet::new();
        visited.insert(origin);

        while let Some((idx, stress)) = queue.pop() {
            if stress < 0.01 { continue; }

            for &neighbor in &self.bonds[idx].neighbors.clone() {
                if visited.contains(&neighbor) { continue; }
                visited.insert(neighbor);

                let attenuation = 1.0 / self.bonds[neighbor].bond_strength;
                let propagated_stress = stress * attenuation;

                self.bonds[neighbor].stress = (self.bonds[neighbor].stress + propagated_stress).min(1.0);

                if propagated_stress > 0.1 {
                    queue.push((neighbor, propagated_stress * 0.5));
                }
            }
        }
    }

    fn update_intensity(&mut self, timestamp: f64, impact: f64) {
        let decay = self.config.decay_beta;
        let alpha = self.config.self_excitation_alpha;

        if let Some(&(last_t, _)) = self.event_history.back() {
            let dt = timestamp - last_t;
            for i in 0..self.current_intensity.len() {
                self.current_intensity[i] *= (-decay * dt).exp();
            }
        }

        let idx = (impact * self.current_intensity.len() as f64) as usize % self.current_intensity.len();
        self.current_intensity[idx] += alpha * impact;

        self.event_history.push_back((timestamp, self.current_intensity.sum()));
        if self.event_history.len() > 10000 {
            self.event_history.pop_front();
        }
    }

    /// Simulate future cascade path using branching process.
    pub fn simulate_cascade(&self, time_horizon: f64, n_simulations: usize) -> CascadeSimulation {
        let mut total_cascades = 0usize;
        let mut max_cascade_size = 0usize;

        for _ in 0..n_simulations {
            let size = self.simulate_single_cascade(time_horizon);
            total_cascades += size;
            max_cascade_size = max_cascade_size.max(size);
        }

        CascadeSimulation {
            mean_cascade_size: total_cascades as f64 / n_simulations as f64,
            max_cascade_size,
            probability_supercritical: if self.branching_ratio_estimate > 1.0 { 1.0 } else { 0.0 },
            expected_duration: time_horizon,
        }
    }

    fn simulate_single_cascade(&self, time_horizon: f64) -> usize {
        let mut count = 1usize;
        let mut current_time = 0.0;
        let mut events = vec![0.0];

        let exp_dist = Exp::new(self.config.decay_beta).unwrap();

        while current_time < time_horizon && !events.is_empty() {
            let event_time = events.remove(0);
            current_time = event_time;

            let n_offspring = if self.branching_ratio_estimate > 1.0 {
                (self.branching_ratio_estimate * 2.0) as usize
            } else {
                (self.branching_ratio_estimate * rand::random::<f64>()) as usize
            };

            for _ in 0..n_offspring {
                let interarrival = exp_dist.pdf(rand::random::<f64>());
                let child_time = current_time + interarrival;
                if child_time < time_horizon {
                    events.push(child_time);
                    count += 1;
                }
            }

            events.sort_by(|a, b| a.partial_cmp(b).unwrap());
        }

        count
    }

    /// Get the "fracture front" — the boundary between intact and fractured bonds.
    pub fn fracture_front(&self) -> Vec<(f64, f64)> {
        self.bonds.iter()
            .filter(|b| b.stress > 0.5 && b.stress < 1.0)
            .map(|b| (b.price_level, b.stress))
            .collect()
    }
}

/// Output metrics from the Hawkes engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HawkesMetrics {
    pub branching_ratio: f64,
    pub intensity: f64,
    pub expected_cascades: f64,
}

/// Result of a cascade simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeSimulation {
    pub mean_cascade_size: f64,
    pub max_cascade_size: usize,
    pub probability_supercritical: f64,
    pub expected_duration: f64,
}
