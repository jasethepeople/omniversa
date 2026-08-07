//! Neural Panic Engine — The "Biological" Layer
//! 
//! Scientific Basis: Hodgkin-Huxley Model (NEURON)
//! Analogy: The market is a global brain; panic is a grand mal seizure.
//! 
//! Simulates agent "firing rates." Agents have "refractory periods" (capital lock-up)
//! and "threshold potentials" (risk tolerance). When narrative stress reaches a 
//! certain voltage, a Depolarization Wave sweeps through the graph.

#![warn(missing_docs)]
#![deny(unsafe_code)]

use nalgebra::{DVector, DMatrix};
use ndarray::{Array1, Array2};
use rayon::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

/// Configuration for the Neural Panic engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralConfig {
    pub n_neurons: usize,
    pub resting_potential: f64,
    pub threshold_potential: f64,
    pub refractory_period_ms: f64,
    pub sodium_conductance: f64,
    pub potassium_conductance: f64,
    pub leak_conductance: f64,
    pub coupling_strength: f64,
    pub sync_threshold: f64,
}

impl Default for NeuralConfig {
    fn default() -> Self {
        Self {
            n_neurons: 10000,
            resting_potential: -65.0,
            threshold_potential: -55.0,
            refractory_period_ms: 2.0,
            sodium_conductance: 120.0,
            potassium_conductance: 36.0,
            leak_conductance: 0.3,
            coupling_strength: 0.01,
            sync_threshold: 0.8,
        }
    }
}

/// A single "neuron" — represents an agent in the market brain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neuron {
    pub id: usize,
    pub voltage: f64,
    pub m: f64,
    pub h: f64,
    pub n_gate: f64,
    pub refractory_end: f64,
    pub risk_tolerance: f64,
    pub capital_locked: f64,
    pub last_fired: f64,
    pub neighbors: Vec<usize>,
}

impl Neuron {
    fn alpha_m(&self, v: f64) -> f64 {
        0.1 * (v + 40.0) / (1.0 - (-(v + 40.0) / 10.0).exp())
    }

    fn beta_m(&self, v: f64) -> f64 {
        4.0 * (-(v + 65.0) / 18.0).exp()
    }

    fn alpha_h(&self, v: f64) -> f64 {
        0.07 * (-(v + 65.0) / 20.0).exp()
    }

    fn beta_h(&self, v: f64) -> f64 {
        1.0 / (1.0 + (-(v + 35.0) / 10.0).exp())
    }

    fn alpha_n(&self, v: f64) -> f64 {
        0.01 * (v + 55.0) / (1.0 - (-(v + 55.0) / 10.0).exp())
    }

    fn beta_n(&self, v: f64) -> f64 {
        0.125 * (-(v + 65.0) / 80.0).exp()
    }

    /// Update state for one time step (Hodgkin-Huxley integration).
    pub fn step(&mut self, dt: f64, input_current: f64, config: &NeuralConfig) {
        let v = self.voltage;

        let dm = self.alpha_m(v) * (1.0 - self.m) - self.beta_m(v) * self.m;
        let dh = self.alpha_h(v) * (1.0 - self.h) - self.beta_h(v) * self.h;
        let dn = self.alpha_n(v) * (1.0 - self.n_gate) - self.beta_n(v) * self.n_gate;

        self.m += dm * dt;
        self.h += dh * dt;
        self.n_gate += dn * dt;

        self.m = self.m.clamp(0.0, 1.0);
        self.h = self.h.clamp(0.0, 1.0);
        self.n_gate = self.n_gate.clamp(0.0, 1.0);

        let g_na = config.sodium_conductance * self.m.powi(3) * self.h;
        let g_k = config.potassium_conductance * self.n_gate.powi(4);
        let g_l = config.leak_conductance;

        let i_na = g_na * (v - 50.0);
        let i_k = g_k * (v - (-77.0));
        let i_l = g_l * (v - (-54.4));

        let dv = (input_current - i_na - i_k - i_l) / 1.0;
        self.voltage += dv * dt;

        let effective_threshold = config.threshold_potential + self.risk_tolerance * 10.0;
        if self.voltage >= effective_threshold && self.refractory_end <= 0.0 {
            self.fire();
        }
    }

    fn fire(&mut self) {
        self.voltage = 30.0;
        self.last_fired = 0.0;
        self.refractory_end = 2.0;
    }
}

/// The Neural Panic engine — simulates the market as a global brain.
#[derive(Debug, Clone)]
pub struct NeuralEngine {
    config: NeuralConfig,
    neurons: Vec<Neuron>,
    firing_history: VecDeque<Vec<usize>>,
    sync_index: f64,
    depolarization_active: bool,
    wave_origin: Option<usize>,
}

impl NeuralEngine {
    /// Initialize the neural engine.
    pub fn new(config: NeuralConfig) -> Result<Self, String> {
        let neurons: Vec<Neuron> = (0..config.n_neurons)
            .map(|i| Neuron {
                id: i,
                voltage: config.resting_potential,
                m: 0.05,
                h: 0.6,
                n_gate: 0.32,
                refractory_end: 0.0,
                risk_tolerance: rand::random::<f64>() * 0.2 - 0.1,
                capital_locked: 0.0,
                last_fired: -1000.0,
                neighbors: vec![],
            })
            .collect();

        let mut engine = Self {
            config: config.clone(),
            neurons,
            firing_history: VecDeque::with_capacity(1000),
            sync_index: 0.0,
            depolarization_active: false,
            wave_origin: None,
        };

        engine.initialize_topology();
        Ok(engine)
    }

    fn initialize_topology(&mut self) {
        let n = self.config.n_neurons;
        for i in 0..n {
            let mut neighbors = Vec::new();
            for j in 1..=5 {
                neighbors.push((i + j) % n);
                neighbors.push((i + n - j) % n);
            }
            if rand::random::<f64>() < 0.1 {
                neighbors.push(rand::random::<usize>() % n);
            }
            self.neurons[i].neighbors = neighbors;
        }
    }

    /// Evaluate neural state given systemic stress.
    pub fn evaluate(&self, stress: &[f64]) -> Result<NeuralMetrics, String> {
        let firing_rate = self.compute_firing_rate();
        let sync_index = self.sync_index;
        let depolarization_wave = self.depolarization_active || sync_index > self.config.sync_threshold;

        Ok(NeuralMetrics {
            mean_firing_rate: firing_rate,
            sync_index,
            depolarization_wave,
        })
    }

    fn compute_firing_rate(&self) -> f64 {
        let recent_firings: usize = self.firing_history.iter().map(|v| v.len()).sum();
        let window_size = self.firing_history.len().max(1);
        recent_firings as f64 / (window_size as f64 * self.config.refractory_period_ms)
    }

    /// Simulate one time step of the neural network.
    pub fn step(&mut self, dt: f64, external_stress: f64) {
        let mut fired_this_step = Vec::new();
        let coupling = self.config.coupling_strength;

        for i in 0..self.neurons.len() {
            let mut synaptic_input = external_stress * 10.0;

            for &neighbor in &self.neurons[i].neighbors.clone() {
                if self.neurons[neighbor].voltage > self.config.threshold_potential {
                    synaptic_input += coupling * 20.0;
                }
            }

            self.neurons[i].step(dt, synaptic_input, &self.config);

            if self.neurons[i].voltage >= 30.0 {
                fired_this_step.push(i);
            }
        }

        self.update_sync_index(&fired_this_step);

        if self.sync_index > self.config.sync_threshold && !self.depolarization_active {
            self.depolarization_active = true;
            self.wave_origin = fired_this_step.first().copied();
        } else if self.sync_index < 0.3 && self.depolarization_active {
            self.depolarization_active = false;
            self.wave_origin = None;
        }

        self.firing_history.push_back(fired_this_step);
        if self.firing_history.len() > 1000 {
            self.firing_history.pop_front();
        }
    }

    fn update_sync_index(&mut self, fired: &[usize]) {
        if fired.len() < 2 {
            self.sync_index *= 0.99;
            return;
        }

        let n = fired.len() as f64;
        let mut sum_cos = 0.0;
        let mut sum_sin = 0.0;

        for &i in fired {
            let phase = (i as f64 / self.config.n_neurons as f64) * 2.0 * std::f64::consts::PI;
            sum_cos += phase.cos();
            sum_sin += phase.sin();
        }

        let r = ((sum_cos / n).powi(2) + (sum_sin / n).powi(2)).sqrt();
        self.sync_index = self.sync_index * 0.9 + r * 0.1;
    }

    /// Predict when the next depolarization wave will occur.
    pub fn predict_wave(&self, horizon_ms: f64) -> WavePrediction {
        if self.depolarization_active {
            return WavePrediction {
                imminent: true,
                estimated_time_ms: 0.0,
                confidence: 1.0,
                affected_neurons: self.neurons.len(),
            };
        }

        let trend = if self.sync_index > 0.5 { 1.0 } else { -1.0 };
        let time_to_threshold = if trend > 0.0 {
            (self.config.sync_threshold - self.sync_index) / (trend * 0.01)
        } else {
            f64::INFINITY
        };

        WavePrediction {
            imminent: time_to_threshold < horizon_ms,
            estimated_time_ms: time_to_threshold,
            confidence: self.sync_index,
            affected_neurons: (self.sync_index * self.neurons.len() as f64) as usize,
        }
    }

    /// Get the "panic voltage map" — which neurons are closest to firing.
    pub fn panic_voltage_map(&self) -> Vec<(usize, f64)> {
        self.neurons.iter()
            .enumerate()
            .map(|(i, n)| (i, n.voltage))
            .filter(|(_, v)| *v > self.config.resting_potential + 5.0)
            .collect()
    }
}

/// Output metrics from the Neural engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralMetrics {
    pub mean_firing_rate: f64,
    pub sync_index: f64,
    pub depolarization_wave: bool,
}

/// Prediction of an impending depolarization wave.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WavePrediction {
    pub imminent: bool,
    pub estimated_time_ms: f64,
    pub confidence: f64,
    pub affected_neurons: usize,
}
