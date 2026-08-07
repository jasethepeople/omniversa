//! Fragility Index — The unified measure of market instability.
//! 
//! This is the number that matters. Not price. Not volume. 
//! The fragility score tells you how close the system is to fracture.

use serde::{Serialize, Deserialize};
use crate::orchestrator::{EngineOutput, EngineKind, EngineMetrics};
use crate::manifold::ManifoldPoint;

/// The master fragility index — a scalar from 0.0 (stable) to 1.0 (fracture imminent).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FragilityIndex {
    pub current_score: f64,
    pub historical_max: f64,
    pub trend: FragilityTrend,
    pub components: FragilityComponents,
    pub last_update_ns: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FragilityComponents {
    pub liquidity_component: f64,
    pub gravity_component: f64,
    pub cascade_component: f64,
    pub neural_component: f64,
    pub narrative_component: f64,
    pub bayesian_component: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FragilityTrend {
    Decreasing,
    Stable,
    Increasing,
    Accelerating,
    Critical,
}

impl Default for FragilityTrend {
    fn default() -> Self { FragilityTrend::Stable }
}

impl FragilityIndex {
    pub fn update(&mut self, point: &ManifoldPoint, outputs: &[(EngineKind, Result<EngineMetrics, String>)]) {
        let mut components = FragilityComponents::default();

        for (kind, result) in outputs {
            if let Ok(metrics) = result {
                match metrics {
                    EngineMetrics::Cfd { cavitation_risk, .. } => {
                        components.liquidity_component = cavitation_risk;
                    }
                    EngineMetrics::Gravity { event_horizon_radius, .. } => {
                        components.gravity_component = event_horizon_radius.min(1.0);
                    }
                    EngineMetrics::Hawkes { expected_cascades, .. } => {
                        components.cascade_component = (expected_cascades / 10.0).min(1.0);
                    }
                    EngineMetrics::Neural { sync_index, depolarization_wave, .. } => {
                        components.neural_component = if *depolarization_wave { 1.0 } else { *sync_index };
                    }
                    EngineMetrics::Narrative { entropy_bits, .. } => {
                        components.narrative_component = (entropy_bits / 8.0).min(1.0);
                    }
                    EngineMetrics::Bayesian { confidence, .. } => {
                        components.bayesian_component = 1.0 - *confidence;
                    }
                    _ => {}
                }
            }
        }

        // Weighted combination — Bayesian gets highest weight as the observer
        self.current_score = 
            components.liquidity_component * 0.15 +
            components.gravity_component * 0.20 +
            components.cascade_component * 0.20 +
            components.neural_component * 0.15 +
            components.narrative_component * 0.10 +
            components.bayesian_component * 0.20;

        if self.current_score > self.historical_max {
            self.historical_max = self.current_score;
        }

        self.trend = self.classify_trend();
        self.components = components;
        self.last_update_ns = point.timestamp_ns;
    }

    fn classify_trend(&self) -> FragilityTrend {
        match self.current_score {
            s if s > 0.9 => FragilityTrend::Critical,
            s if s > 0.7 => FragilityTrend::Accelerating,
            s if s > 0.5 => FragilityTrend::Increasing,
            s if s > 0.3 => FragilityTrend::Stable,
            _ => FragilityTrend::Decreasing,
        }
    }

    pub fn snapshot(&self) -> crate::FragilitySnapshot {
        crate::FragilitySnapshot {
            timestamp_ns: self.last_update_ns,
            systemic_risk_field: crate::manifold::RiskField::new(
                (32, 32, 32),
                ((0.0, 1.0), (0.0, 1.0), (0.0, 1.0))
            ),
            cascade_probability_tensor: vec![self.components.cascade_component; 6],
            volatility_weather_map: vec![],
            liquidity_collapse_radar: vec![],
            narrative_shock_index: self.components.narrative_component,
            phase_space_position: ManifoldPoint::new(self.last_update_ns),
            fragility_score: self.current_score,
            engines_status: vec![],
        }
    }
}

/// A tensor representing cascade probability across the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeProbabilityVector {
    pub probabilities: Vec<f64>,
    pub node_ids: Vec<u64>,
    pub total_expected_cascades: f64,
}

/// The systemic risk tensor — directional field of stress propagation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemicRiskTensor {
    pub field: Vec<Vec<f64>>,
    pub divergence: f64,
    pub curl: f64,
}
