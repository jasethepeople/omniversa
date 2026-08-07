//! Gravity Whale Engine — The "Spacetime" Layer
//! 
//! Scientific Basis: N-Body Simulations (GADGET-4 analogy)
//! Analogy: Large capital pools are stellar masses.
//! 
//! Models the "gravitational pull" of massive limit orders. Price is a particle
//! moving through a warped manifold. Whales create Event Horizons.

#![warn(missing_docs)]
#![deny(unsafe_code)]

use nalgebra::{Vector3, Matrix3};
use ndarray::{Array1, Array2};
use rayon::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Configuration for the Gravity engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GravityConfig {
    pub gravitational_constant: f64,
    pub softening_length: f64,
    pub event_horizon_threshold: f64,
    pub max_whale_mass: f64,
    pub time_step: f64,
}

impl Default for GravityConfig {
    fn default() -> Self {
        Self {
            gravitational_constant: 6.674e-5,
            softening_length: 0.1,
            event_horizon_threshold: 0.5,
            max_whale_mass: 1e9,
            time_step: 0.01,
        }
    }
}

/// A "celestial body" in the market — represents a whale's capital pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleBody {
    pub id: u64,
    pub position: Vector3<f64>,
    pub velocity: Vector3<f64>,
    pub mass: f64,
    pub radius_of_influence: f64,
    pub liquidation_cluster: Vec<f64>,
}

/// The Gravity engine — N-body simulation of whale capital.
#[derive(Debug, Clone)]
pub struct GravityEngine {
    config: GravityConfig,
    whales: Vec<WhaleBody>,
    price_particle: Vector3<f64>,
    price_velocity: Vector3<f64>,
    metric_tensor: Matrix3<f64>,
    manifold_curvature: f64,
}

impl GravityEngine {
    /// Initialize the gravity engine.
    pub fn new(config: GravityConfig) -> Result<Self, String> {
        Ok(Self {
            config: config.clone(),
            whales: Vec::new(),
            price_particle: Vector3::zeros(),
            price_velocity: Vector3::zeros(),
            metric_tensor: Matrix3::identity(),
            manifold_curvature: 0.0,
        })
    }

    /// Evaluate gravitational effects given systemic stress.
    pub fn evaluate(&self, stress: &[f64]) -> Result<GravityMetrics, String> {
        let total_mass: f64 = self.whales.iter().map(|w| w.mass).sum();

        let gravitational_pull = self.compute_total_gravity();
        let event_horizon_radius = self.compute_event_horizon(total_mass);
        let manifold_warp = self.compute_manifold_warp();

        Ok(GravityMetrics {
            gravitational_pull,
            event_horizon_radius,
            manifold_warp,
        })
    }

    fn compute_total_gravity(&self) -> f64 {
        let mut total_force = 0.0;

        for whale in &self.whales {
            let r = (whale.position - self.price_particle).norm();
            let softened_r = (r * r + self.config.softening_length * self.config.softening_length).sqrt();
            let force = self.config.gravitational_constant * whale.mass / (softened_r * softened_r);
            total_force += force;
        }

        total_force
    }

    fn compute_event_horizon(&self, total_mass: f64) -> f64 {
        let c = 3e8;
        2.0 * self.config.gravitational_constant * total_mass / (c * c)
    }

    fn compute_manifold_warp(&self) -> f64 {
        let mut curvature = 0.0;
        for i in 0..3 {
            for j in 0..3 {
                curvature += self.metric_tensor[(i, j)].abs();
            }
        }
        curvature / 9.0
    }

    /// Add a new whale to the simulation.
    pub fn add_whale(&mut self, id: u64, capital: f64, position: [f64; 3], liquidation_levels: Vec<f64>) {
        let mass = capital.min(self.config.max_whale_mass);
        let radius = (self.config.gravitational_constant * mass).sqrt();

        self.whales.push(WhaleBody {
            id,
            position: Vector3::new(position[0], position[1], position[2]),
            velocity: Vector3::zeros(),
            mass,
            radius_of_influence: radius,
            liquidation_cluster: liquidation_levels,
        });

        self.update_metric_tensor();
    }

    fn update_metric_tensor(&mut self) {
        let mut phi = 0.0;
        for whale in &self.whales {
            let r = (whale.position - self.price_particle).norm();
            phi -= self.config.gravitational_constant * whale.mass / r.max(1e-10);
        }

        for i in 0..3 {
            for j in 0..3 {
                self.metric_tensor[(i, j)] = if i == j { 1.0 + 2.0 * phi } else { 0.0 };
            }
        }
    }

    /// Simulate one time step of the N-body system.
    pub fn step(&mut self) {
        let dt = self.config.time_step;

        let mut acceleration = Vector3::zeros();
        for whale in &self.whales {
            let r_vec = whale.position - self.price_particle;
            let r = r_vec.norm();
            let softened_r = (r * r + self.config.softening_length * self.config.softening_length).sqrt();
            let force_magnitude = self.config.gravitational_constant * whale.mass / (softened_r * softened_r * softened_r);
            acceleration += r_vec * force_magnitude;
        }

        self.price_velocity += acceleration * dt;
        self.price_particle += self.price_velocity * dt;

        for whale in &mut self.whales {
            whale.position += whale.velocity * dt * 0.1;
        }

        self.update_metric_tensor();
    }

    /// Predict if price will be captured by a whale's event horizon.
    pub fn predict_capture(&self, horizon_ns: u64) -> Vec<(u64, f64)> {
        let mut captures = Vec::new();

        for whale in &self.whales {
            let distance = (whale.position - self.price_particle).norm();
            let approach_velocity = self.price_velocity.norm();

            if approach_velocity > 1e-10 {
                let time_to_horizon = (distance - whale.radius_of_influence) / approach_velocity;
                if time_to_horizon > 0.0 && time_to_horizon < horizon_ns as f64 {
                    captures.push((whale.id, time_to_horizon));
                }
            }
        }

        captures
    }

    /// Compute the "escape velocity" required to break free from current gravity well.
    pub fn escape_velocity(&self) -> f64 {
        let total_mass: f64 = self.whales.iter().map(|w| w.mass).sum();
        let r = self.whales.iter()
            .map(|w| (w.position - self.price_particle).norm())
            .fold(f64::INFINITY, f64::min);

        (2.0 * self.config.gravitational_constant * total_mass / r.max(1e-10)).sqrt()
    }
}

/// Output metrics from the Gravity engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GravityMetrics {
    pub gravitational_pull: f64,
    pub event_horizon_radius: f64,
    pub manifold_warp: f64,
}
