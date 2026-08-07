//! Liquidity CFD Engine — The "Viscosity" Layer
//! 
//! Scientific Basis: Navier-Stokes Equations (OpenFOAM analogy)
//! Analogy: Liquidity is a non-Newtonian fluid.
//! 
//! Models the "flow" of capital through order books. HFT creates vortices
//! and laminar flow. When liquidity vanishes, "viscosity" increases.
//! Cavitation (price gaps) occurs when sell-side pressure exceeds available bids.

#![warn(missing_docs)]
#![deny(unsafe_code)]

use nalgebra::{DVector, DMatrix};
use ndarray::{Array1, Array2, Array3, s};
use rayon::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

/// Configuration for the CFD engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfdConfig {
    pub grid_resolution: usize,
    pub time_step: f64,
    pub kinematic_viscosity: f64,
    pub density: f64,
    pub max_velocity: f64,
    pub cavitation_threshold: f64,
}

impl Default for CfdConfig {
    fn default() -> Self {
        Self {
            grid_resolution: 128,
            time_step: 0.001,
            kinematic_viscosity: 0.01,
            density: 1000.0,
            max_velocity: 10.0,
            cavitation_threshold: 0.3,
        }
    }
}

/// The CFD engine simulates liquidity as a fluid.
#[derive(Debug, Clone)]
pub struct CfdEngine {
    config: CfdConfig,
    velocity_field: Array3<f64>,
    pressure_field: Array2<f64>,
    viscosity_field: Array2<f64>,
    order_book_depth: VecDeque<Vec<(f64, f64)>>,
    tick_history: VecDeque<f64>,
}

impl CfdEngine {
    /// Initialize the CFD engine with configuration.
    pub fn new(config: CfdConfig) -> Result<Self, String> {
        let n = config.grid_resolution;
        Ok(Self {
            config: config.clone(),
            velocity_field: Array3::zeros((n, n, 3)),
            pressure_field: Array2::zeros((n, n)),
            viscosity_field: Array2::from_elem((n, n), config.kinematic_viscosity),
            order_book_depth: VecDeque::with_capacity(1000),
            tick_history: VecDeque::with_capacity(10000),
        })
    }

    /// Evaluate the current liquidity state given systemic stress vector.
    pub fn evaluate(&self, stress: &[f64]) -> Result<CfdMetrics, String> {
        let n = self.config.grid_resolution;

        // Compute vorticity (measure of turbulence in liquidity flow)
        let vorticity = self.compute_vorticity();

        // Compute effective viscosity (increases as liquidity dries up)
        let mean_stress = if stress.is_empty() { 0.0 } else { 
            stress.iter().sum::<f64>() / stress.len() as f64 
        };
        let effective_viscosity = self.config.kinematic_viscosity * (1.0 + mean_stress * 10.0);

        // Detect cavitation zones (price gaps)
        let cavitation_risk = self.detect_cavitation();

        // Compute Reynolds number analog
        let reynolds = self.compute_reynolds(effective_viscosity);

        Ok(CfdMetrics {
            viscosity: effective_viscosity,
            vorticity,
            cavitation_risk,
            reynolds,
        })
    }

    fn compute_vorticity(&self) -> f64 {
        let n = self.config.grid_resolution;
        let mut vorticity = 0.0;

        for i in 1..(n-1) {
            for j in 1..(n-1) {
                let dudy = (self.velocity_field[[i, j+1, 0]] - self.velocity_field[[i, j-1, 0]]) / 2.0;
                let dvdx = (self.velocity_field[[i+1, j, 1]] - self.velocity_field[[i-1, j, 1]]) / 2.0;
                vorticity += (dvdx - dudy).abs();
            }
        }

        vorticity / ((n-2) * (n-2)) as f64
    }

    fn detect_cavitation(&self) -> f64 {
        let n = self.config.grid_resolution;
        let mut cavitation_zones = 0usize;

        for i in 0..n {
            for j in 0..n {
                if self.pressure_field[[i, j]] < self.config.cavitation_threshold {
                    cavitation_zones += 1;
                }
            }
        }

        cavitation_zones as f64 / (n * n) as f64
    }

    fn compute_reynolds(&self, viscosity: f64) -> f64 {
        let mean_velocity = self.velocity_field.iter().sum::<f64>() / self.velocity_field.len() as f64;
        let characteristic_length = self.config.grid_resolution as f64;

        mean_velocity * characteristic_length / viscosity.max(1e-10)
    }

    /// Update the velocity field with new tick data.
    pub fn ingest_tick(&mut self, bid_depth: &[(f64, f64)], ask_depth: &[(f64, f64)]) {
        self.order_book_depth.push_back(bid_depth.to_vec());
        if self.order_book_depth.len() > 1000 {
            self.order_book_depth.pop_front();
        }

        self.map_orderbook_to_field(bid_depth, ask_depth);
        self.solve_navier_stokes_step();
    }

    fn map_orderbook_to_field(&mut self, bid_depth: &[(f64, f64)], ask_depth: &[(f64, f64)]) {
        let n = self.config.grid_resolution;

        let all_prices: Vec<f64> = bid_depth.iter().chain(ask_depth.iter()).map(|(p, _)| *p).collect();
        if all_prices.is_empty() { return; }

        let min_p = all_prices.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_p = all_prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = (max_p - min_p).max(1e-10);

        for (price, volume) in bid_depth {
            let x = ((price - min_p) / range * (n - 1) as f64) as usize;
            let y = (volume.ln_1p() * 10.0) as usize % n;
            if x < n && y < n {
                self.velocity_field[[x, y, 0]] += volume * 0.1;
            }
        }

        for (price, volume) in ask_depth {
            let x = ((price - min_p) / range * (n - 1) as f64) as usize;
            let y = (volume.ln_1p() * 10.0) as usize % n;
            if x < n && y < n {
                self.velocity_field[[x, y, 1]] -= volume * 0.1;
            }
        }
    }

    fn solve_navier_stokes_step(&mut self) {
        let n = self.config.grid_resolution;
        let dt = self.config.time_step;
        let nu = self.config.kinematic_viscosity;

        let mut new_velocity = self.velocity_field.clone();

        for i in 1..(n-1) {
            for j in 1..(n-1) {
                for k in 0..3 {
                    let u = self.velocity_field[[i, j, 0]];
                    let v = self.velocity_field[[i, j, 1]];

                    let src_i = (i as f64 - u * dt).max(1.0).min((n-2) as f64) as usize;
                    let src_j = (j as f64 - v * dt).max(1.0).min((n-2) as f64) as usize;

                    let advected = self.velocity_field[[src_i, src_j, k]];

                    let laplacian = 
                        self.velocity_field[[i+1, j, k]] +
                        self.velocity_field[[i-1, j, k]] +
                        self.velocity_field[[i, j+1, k]] +
                        self.velocity_field[[i, j-1, k]] -
                        4.0 * self.velocity_field[[i, j, k]];

                    new_velocity[[i, j, k]] = advected + nu * laplacian * dt;
                }
            }
        }

        self.velocity_field = new_velocity;
    }

    /// Compute the "slippage forecast" — expected price impact for a given order size.
    pub fn slippage_forecast(&self, order_size: f64, side: OrderSide) -> f64 {
        let viscosity = self.compute_vorticity();
        let base_slippage = order_size * viscosity * 0.001;

        match side {
            OrderSide::Buy => base_slippage,
            OrderSide::Sell => base_slippage * 1.5,
        }
    }
}

/// Output metrics from the CFD engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfdMetrics {
    pub viscosity: f64,
    pub vorticity: f64,
    pub cavitation_risk: f64,
    pub reynolds: f64,
}

/// Order side for slippage calculation.
#[derive(Debug, Clone, Copy)]
pub enum OrderSide { Buy, Sell }
