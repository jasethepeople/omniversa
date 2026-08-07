//! Engine Orchestrator — The hexagonal coupling controller.
//! 
//! Manages the six engines as a unified system, handling data flow,
//! synchronization, and the Bayesian update cycle.

use tokio::sync::{RwLock, mpsc};
use std::sync::Arc;
use tracing::{info, debug, warn};
use std::time::{Duration, Instant};

use crate::{
    manifold::PhaseSpaceManifold,
    topology::UnifiedGraphTopology,
    fragility::FragilityIndex,
    EngineConfig, MarketEvent, OmniversaError,
};

/// The six engine kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineKind {
    LiquidityCfd,
    GravityWhale,
    CascadeHawkes,
    NeuralPanic,
    NarrativeNlp,
    BayesianRisk,
}

/// Output from any engine.
#[derive(Debug, Clone)]
pub struct EngineOutput {
    pub kind: EngineKind,
    pub timestamp_ns: u64,
    pub metrics: EngineMetrics,
}

#[derive(Debug, Clone)]
pub enum EngineMetrics {
    Cfd { viscosity: f64, vorticity: f64, cavitation_risk: f64 },
    Gravity { gravitational_pull: f64, event_horizon_radius: f64, manifold_warp: f64 },
    Hawkes { branching_ratio: f64, intensity: f64, expected_cascades: f64 },
    Neural { mean_firing_rate: f64, sync_index: f64, depolarization_wave: bool },
    Narrative { pressure: f64, moisture: f64, pathogen_count: usize, entropy_bits: f64 },
    Bayesian { posterior_entropy: f64, dark_matter_estimate: f64, confidence: f64 },
}

/// The orchestrator that couples all six engines.
pub struct EngineOrchestrator {
    topology: Arc<RwLock<UnifiedGraphTopology>>,
    manifold: Arc<RwLock<PhaseSpaceManifold>>,
    fragility: Arc<RwLock<FragilityIndex>>,
    config: EngineConfig,

    // Engine instances
    cfd_engine: liquidity_cfd::CfdEngine,
    gravity_engine: gravity_whale::GravityEngine,
    hawkes_engine: cascade_hawkes::HawkesEngine,
    neural_engine: neural_panic::NeuralEngine,
    narrative_engine: narrative_nlp::NarrativeEngine,
    bayesian_engine: bayesian_risk::BayesianEngine,

    // Event channels
    event_tx: mpsc::Sender<MarketEvent>,
    event_rx: mpsc::Receiver<MarketEvent>,

    last_tick: Instant,
    tick_interval_ms: u64,
}

impl EngineOrchestrator {
    pub async fn new(
        topology: Arc<RwLock<UnifiedGraphTopology>>,
        manifold: Arc<RwLock<PhaseSpaceManifold>>,
        fragility: Arc<RwLock<FragilityIndex>>,
        config: EngineConfig,
    ) -> Result<Self, OmniversaError> {
        let (event_tx, event_rx) = mpsc::channel(10000);

        let cfd_engine = liquidity_cfd::CfdEngine::new(config.cfd.clone())
            .map_err(|e| OmniversaError::Engine(format!("CFD init: {}", e)))?;

        let gravity_engine = gravity_whale::GravityEngine::new(config.gravity.clone())
            .map_err(|e| OmniversaError::Engine(format!("Gravity init: {}", e)))?;

        let hawkes_engine = cascade_hawkes::HawkesEngine::new(config.hawkes.clone())
            .map_err(|e| OmniversaError::Engine(format!("Hawkes init: {}", e)))?;

        let neural_engine = neural_panic::NeuralEngine::new(config.neural.clone())
            .map_err(|e| OmniversaError::Engine(format!("Neural init: {}", e)))?;

        let narrative_engine = narrative_nlp::NarrativeEngine::new(config.narrative.clone())
            .map_err(|e| OmniversaError::Engine(format!("Narrative init: {}", e)))?;

        let bayesian_engine = bayesian_risk::BayesianEngine::new(config.bayesian.clone())
            .map_err(|e| OmniversaError::Engine(format!("Bayesian init: {}", e)))?;

        info!("All six engines initialized and ready for hexagonal coupling");

        Ok(Self {
            topology,
            manifold,
            fragility,
            config,
            cfd_engine,
            gravity_engine,
            hawkes_engine,
            neural_engine,
            narrative_engine,
            bayesian_engine,
            event_tx,
            event_rx,
            last_tick: Instant::now(),
            tick_interval_ms: 100, // 100ms default tick
        })
    }

    /// Main tick loop — runs all engines and updates the manifold.
    pub async fn tick(&self) {
        let start = Instant::now();

        // 1. Read topology snapshot
        let topo_snapshot = {
            let topo = self.topology.read().await;
            topo.systemic_stress_vector()
        };

        // 2. Run all six engines in parallel using rayon
        let outputs = tokio::task::spawn_blocking({
            let cfd = self.cfd_engine.clone();
            let gravity = self.gravity_engine.clone();
            let hawkes = self.hawkes_engine.clone();
            let neural = self.neural_engine.clone();
            let narrative = self.narrative_engine.clone();
            let bayesian = self.bayesian_engine.clone();
            let stress = topo_snapshot.clone();

            move || {
                use rayon::prelude::*;

                vec![
                    (EngineKind::LiquidityCfd, cfd.evaluate(&stress)),
                    (EngineKind::GravityWhale, gravity.evaluate(&stress)),
                    (EngineKind::CascadeHawkes, hawkes.evaluate(&stress)),
                    (EngineKind::NeuralPanic, neural.evaluate(&stress)),
                    (EngineKind::NarrativeNlp, narrative.evaluate(&stress)),
                    (EngineKind::BayesianRisk, bayesian.evaluate(&stress)),
                ]
            }
        }).await.unwrap_or_default();

        // 3. Convert outputs to manifold point
        let mut point = crate::manifold::ManifoldPoint::new(
            start.elapsed().as_nanos() as u64
        );

        for (kind, result) in outputs {
            match result {
                Ok(metrics) => {
                    match metrics {
                        EngineMetrics::Cfd { viscosity, .. } => {
                            point.liquidity_viscosity = viscosity;
                        }
                        EngineMetrics::Gravity { gravitational_pull, .. } => {
                            point.whale_gravity = gravitational_pull;
                        }
                        EngineMetrics::Hawkes { branching_ratio, .. } => {
                            point.cascade_stress = branching_ratio;
                        }
                        EngineMetrics::Neural { sync_index, .. } => {
                            point.neural_voltage = sync_index;
                        }
                        EngineMetrics::Narrative { pressure, .. } => {
                            point.narrative_pressure = pressure;
                        }
                        EngineMetrics::Bayesian { posterior_entropy, .. } => {
                            point.bayesian_entropy = posterior_entropy;
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    warn!("Engine {:?} error: {}", kind, e);
                }
            }
        }

        // 4. Update manifold
        {
            let mut manifold = self.manifold.write().await;
            manifold.ingest(point);
        }

        // 5. Update fragility index
        {
            let mut frag = self.fragility.write().await;
            frag.update(&point, &outputs);
        }

        let elapsed = start.elapsed();
        if elapsed > Duration::from_millis(self.tick_interval_ms) {
            warn!("Tick exceeded interval: {:?}", elapsed);
        }
    }

    pub async fn force_evaluation(&self) -> Result<(), OmniversaError> {
        self.tick().await;
        Ok(())
    }

    pub fn event_sender(&self) -> mpsc::Sender<MarketEvent> {
        self.event_tx.clone()
    }
}
