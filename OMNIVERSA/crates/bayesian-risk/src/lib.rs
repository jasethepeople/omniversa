//! Bayesian Risk Engine — The "Observer" Layer
//! 
//! Scientific Basis: Hamiltonian Monte Carlo (Stan)
//! Analogy: The Quantum State Observer.
//! 
//! The master orchestrator. Takes outputs of the other 5 engines and updates
//! the Global Probability Distribution. Continuously asks: "Given the current
//! CFD turbulence and Whale gravity, what is the hidden 'dark matter'
//! (hidden leverage) in the system?"

#![warn(missing_docs)]
#![deny(unsafe_code)]

use nalgebra::{DVector, DMatrix, Cholesky};
use ndarray::{Array1, Array2, Axis};
use rayon::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use rand::Rng;

/// Configuration for the Bayesian engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BayesianConfig {
    pub n_chains: usize,
    pub n_samples: usize,
    pub step_size: f64,
    pub max_tree_depth: usize,
    pub prior_mean: f64,
    pub prior_std: f64,
    pub n_parameters: usize,
}

impl Default for BayesianConfig {
    fn default() -> Self {
        Self {
            n_chains: 4,
            n_samples: 2000,
            step_size: 0.1,
            max_tree_depth: 10,
            prior_mean: 0.0,
            prior_std: 1.0,
            n_parameters: 6,
        }
    }
}

/// A single HMC sample from the posterior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HMCSample {
    pub parameters: Vec<f64>,
    pub log_probability: f64,
    pub gradient: Vec<f64>,
    pub accepted: bool,
}

/// The Bayesian engine — Hamiltonian Monte Carlo for hidden leverage inference.
#[derive(Debug, Clone)]
pub struct BayesianEngine {
    config: BayesianConfig,
    posterior_samples: VecDeque<HMCSample>,
    current_position: Vec<f64>,
    covariance_estimate: Array2<f64>,
    dark_matter_estimate: f64,
    global_entropy: f64,
}

impl BayesianEngine {
    /// Initialize the Bayesian engine.
    pub fn new(config: BayesianConfig) -> Result<Self, String> {
        let mut rng = rand::thread_rng();
        let initial_position: Vec<f64> = (0..config.n_parameters)
            .map(|_| rng.gen::<f64>() * config.prior_std + config.prior_mean)
            .collect();

        Ok(Self {
            config: config.clone(),
            posterior_samples: VecDeque::with_capacity(config.n_samples * config.n_chains),
            current_position: initial_position,
            covariance_estimate: Array2::eye(config.n_parameters),
            dark_matter_estimate: 0.0,
            global_entropy: 1.0,
        })
    }

    /// Evaluate Bayesian state given engine outputs.
    pub fn evaluate(&self, stress: &[f64]) -> Result<BayesianMetrics, String> {
        let mean_stress = if stress.is_empty() { 0.0 } else {
            stress.iter().sum::<f64>() / stress.len() as f64
        };

        let posterior_entropy = self.compute_posterior_entropy();
        let dark_matter = self.dark_matter_estimate * (1.0 + mean_stress);
        let confidence = 1.0 / (1.0 + posterior_entropy);

        Ok(BayesianMetrics {
            posterior_entropy,
            dark_matter_estimate: dark_matter,
            confidence,
        })
    }

    fn compute_posterior_entropy(&self) -> f64 {
        if self.posterior_samples.is_empty() { return 1.0; }

        let n = self.posterior_samples.len();
        let params: Vec<Vec<f64>> = self.posterior_samples.iter()
            .map(|s| s.parameters.clone())
            .collect();

        let dim = self.config.n_parameters;
        let mut mean = vec![0.0; dim];
        for p in &params {
            for i in 0..dim {
                mean[i] += p[i];
            }
        }
        for m in &mut mean {
            *m /= n as f64;
        }

        let mut cov = Array2::zeros((dim, dim));
        for p in &params {
            for i in 0..dim {
                for j in 0..dim {
                    cov[[i, j]] += (p[i] - mean[i]) * (p[j] - mean[j]);
                }
            }
        }
        cov /= n as f64;

        let det = self.approx_determinant(&cov);
        0.5 * (dim as f64 * (2.0 * std::f64::consts::PI * std::f64::consts::E).ln() + det.ln())
    }

    fn approx_determinant(&self, matrix: &Array2<f64>) -> f64 {
        let mut det = 1.0;
        for i in 0..matrix.shape()[0] {
            det *= matrix[[i, i]].max(1e-10);
        }
        det
    }

    /// Run one HMC step to update the posterior.
    pub fn hmc_step(&mut self, engine_outputs: &[f64]) {
        let dim = self.config.n_parameters;
        let mut rng = rand::thread_rng();

        let mut momentum: Vec<f64> = (0..dim)
            .map(|_| rng.gen::<f64>() * self.config.prior_std)
            .collect();

        let current_hamiltonian = self.hamiltonian(&self.current_position, &momentum, engine_outputs);

        let mut proposed_position = self.current_position.clone();
        let mut proposed_momentum = momentum.clone();

        let epsilon = self.config.step_size;
        let n_steps = 2usize.pow(self.config.max_tree_depth as u32);

        let grad = self.gradient(&proposed_position, engine_outputs);
        for i in 0..dim {
            proposed_momentum[i] -= 0.5 * epsilon * grad[i];
        }

        for _ in 0..n_steps {
            for i in 0..dim {
                proposed_position[i] += epsilon * proposed_momentum[i];
            }

            let grad = self.gradient(&proposed_position, engine_outputs);
            for i in 0..dim {
                proposed_momentum[i] -= epsilon * grad[i];
            }
        }

        let grad = self.gradient(&proposed_position, engine_outputs);
        for i in 0..dim {
            proposed_momentum[i] -= 0.5 * epsilon * grad[i];
        }

        let proposed_hamiltonian = self.hamiltonian(&proposed_position, &proposed_momentum, engine_outputs);
        let delta_h = proposed_hamiltonian - current_hamiltonian;

        let accepted = if delta_h < 0.0 {
            true
        } else {
            rng.gen::<f64>() < (-delta_h).exp()
        };

        if accepted {
            self.current_position = proposed_position;
        }

        let log_prob = self.log_posterior(&self.current_position, engine_outputs);
        self.posterior_samples.push_back(HMCSample {
            parameters: self.current_position.clone(),
            log_probability: log_prob,
            gradient: grad,
            accepted,
        });

        if self.posterior_samples.len() > self.config.n_samples * self.config.n_chains {
            self.posterior_samples.pop_front();
        }

        self.update_dark_matter();
    }

    fn hamiltonian(&self, position: &[f64], momentum: &[f64], observations: &[f64]) -> f64 {
        let potential = -self.log_posterior(position, observations);
        let kinetic: f64 = momentum.iter().map(|m| m * m).sum();
        potential + 0.5 * kinetic
    }

    fn log_posterior(&self, position: &[f64], observations: &[f64]) -> f64 {
        let log_prior: f64 = position.iter()
            .map(|p| {
                let diff = p - self.config.prior_mean;
                -0.5 * diff * diff / (self.config.prior_std * self.config.prior_std)
            })
            .sum();

        let log_likelihood: f64 = if observations.len() == position.len() {
            position.iter().zip(observations.iter())
                .map(|(p, o)| {
                    let diff = p - o;
                    -0.5 * diff * diff
                })
                .sum()
        } else {
            0.0
        };

        log_prior + log_likelihood
    }

    fn gradient(&self, position: &[f64], observations: &[f64]) -> Vec<f64> {
        let dim = position.len();
        let mut grad = vec![0.0; dim];

        for i in 0..dim {
            grad[i] += -(position[i] - self.config.prior_mean) / (self.config.prior_std * self.config.prior_std);
        }

        if observations.len() == position.len() {
            for i in 0..dim {
                grad[i] += -(position[i] - observations[i]);
            }
        }

        grad
    }

    fn update_dark_matter(&mut self) {
        if self.posterior_samples.is_empty() { return; }

        let recent: Vec<_> = self.posterior_samples.iter().rev().take(100).collect();
        let mean_log_prob: f64 = recent.iter().map(|s| s.log_probability).sum::<f64>() / recent.len() as f64;

        self.dark_matter_estimate = (-mean_log_prob).exp().min(10.0);
    }

    /// Get the posterior mean estimate.
    pub fn posterior_mean(&self) -> Vec<f64> {
        if self.posterior_samples.is_empty() {
            return vec![0.0; self.config.n_parameters];
        }

        let dim = self.config.n_parameters;
        let mut mean = vec![0.0; dim];
        let n = self.posterior_samples.len();

        for sample in &self.posterior_samples {
            for i in 0..dim {
                mean[i] += sample.parameters[i];
            }
        }

        for m in &mut mean {
            *m /= n as f64;
        }

        mean
    }

    /// Compute credible intervals for parameters.
    pub fn credible_intervals(&self, level: f64) -> Vec<(f64, f64)> {
        if self.posterior_samples.is_empty() {
            return vec![(0.0, 0.0); self.config.n_parameters];
        }

        let dim = self.config.n_parameters;
        let mut intervals = Vec::with_capacity(dim);

        for i in 0..dim {
            let mut values: Vec<f64> = self.posterior_samples.iter()
                .map(|s| s.parameters[i])
                .collect();
            values.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let lower_idx = ((1.0 - level) / 2.0 * values.len() as f64) as usize;
            let upper_idx = ((1.0 + level) / 2.0 * values.len() as f64) as usize;

            intervals.push((
                values.get(lower_idx).copied().unwrap_or(0.0),
                values.get(upper_idx.min(values.len() - 1)).copied().unwrap_or(0.0),
            ));
        }

        intervals
    }
}

/// Output metrics from the Bayesian engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BayesianMetrics {
    pub posterior_entropy: f64,
    pub dark_matter_estimate: f64,
    pub confidence: f64,
}
