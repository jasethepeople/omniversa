//! OMNIVERSA Core — The Multiphysics Synthetic Market Sovereign
//! 
//! This crate is the central orchestrator for the six-engine hexagonal coupling.
//! It implements the Unified Graph Topology (UGT), the Phase Space Manifold,
//! and the real-time fragility dashboard.
//!
//! # Architecture
//! ```text
//!                    ┌─────────────────┐
//!                    │  BAYESIAN RISK  │
//!                    │   (Observer)    │
//!                    └────────┬────────┘
//!                             │ HMC Updates
//!        ┌────────────────────┼────────────────────┐
//!        │                    │                    │
//!   ┌────▼────┐        ┌─────▼─────┐        ┌─────▼─────┐
//!   │ LIQUIDITY│        │ GRAVITY  │        │ CASCADE  │
//!   │   CFD    │        │  WHALE   │        │  HAWKES  │
//!   │(Viscosity)│       │(Spacetime)│       │(Seismology)│
//!   └────┬────┘        └─────┬─────┘        └─────┬─────┘
//!        │                   │                    │
//!        └───────────────────┼────────────────────┘
//!                            │
//!                    ┌─────────▼─────────┐
//!                    │   NEURAL PANIC  │
//!                    │  (Biological)   │
//!                    └────────┬────────┘
//!                             │
//!                    ┌────────▼────────┐
//!                    │  NARRATIVE NLP  │
//!                    │  (Atmospheric)  │
//!                    └─────────────────┘
//! ```

#![warn(missing_docs)]
#![deny(unsafe_code)]
#![feature(async_closure)]

pub mod manifold;
pub mod topology;
pub mod orchestrator;
pub mod fragility;
pub mod telemetry;
pub mod consensus;

pub use manifold::{PhaseSpaceManifold, ManifoldPoint, RiskField};
pub use topology::{UnifiedGraphTopology, NodeId, EdgeId, CreditRelation};
pub use orchestrator::{EngineOrchestrator, EngineOutput, EngineKind};
pub use fragility::{FragilityIndex, SystemicRiskTensor, CascadeProbabilityVector};

use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::{info, warn, error};

/// The sovereign engine — single entry point for all market thermodynamic analysis.
pub struct OmniversaEngine {
    topology: Arc<RwLock<UnifiedGraphTopology>>,
    orchestrator: EngineOrchestrator,
    manifold: Arc<RwLock<PhaseSpaceManifold>>,
    fragility: Arc<RwLock<FragilityIndex>>,
    telemetry: telemetry::TelemetryCollector,
    consensus: consensus::SovereignConsensus,
    shutdown: broadcast::Sender<()>,
}

impl OmniversaEngine {
    /// Initialize the sovereign engine with the UGT graph.
    pub async fn new(
        initial_nodes: usize,
        config: EngineConfig,
    ) -> Result<Self, OmniversaError> {
        let topology = Arc::new(RwLock::new(
            UnifiedGraphTopology::with_capacity(initial_nodes)
        ));

        let manifold = Arc::new(RwLock::new(PhaseSpaceManifold::new(6)));
        let fragility = Arc::new(RwLock::new(FragilityIndex::default()));
        let (shutdown_tx, _) = broadcast::channel(1);

        let orchestrator = EngineOrchestrator::new(
            topology.clone(),
            manifold.clone(),
            fragility.clone(),
            config,
        ).await?;

        info!("OMNIVERSA Sovereign Engine v1.0.0 initialized");
        info!("Market Thermodynamics active. Price is a lagging indicator.");

        Ok(Self {
            topology,
            orchestrator,
            manifold,
            fragility,
            telemetry: telemetry::TelemetryCollector::new(),
            consensus: consensus::SovereignConsensus::new(),
            shutdown: shutdown_tx,
        })
    }

    /// Start the hexagonal engine coupling loop.
    pub async fn run(&self) -> Result<(), OmniversaError> {
        let mut rx = self.shutdown.subscribe();

        loop {
            tokio::select! {
                _ = rx.recv() => {
                    info!("Sovereign shutdown signal received. Graceful termination.");
                    break Ok(());
                }
                _ = self.orchestrator.tick() => {
                    self.telemetry.record_tick().await;
                }
            }
        }
    }

    /// Inject a market event into the UGT topology.
    pub async fn inject_event(&self, event: MarketEvent) -> Result<(), OmniversaError> {
        let mut topo = self.topology.write().await;
        topo.process_event(event).await?;
        drop(topo);

        // Trigger immediate re-evaluation
        self.orchestrator.force_evaluation().await?;
        Ok(())
    }

    /// Get current fragility snapshot.
    pub async fn fragility_snapshot(&self) -> FragilitySnapshot {
        let frag = self.fragility.read().await;
        frag.snapshot()
    }

    /// Shutdown the engine gracefully.
    pub fn shutdown(&self) {
        let _ = self.shutdown.send(());
    }
}

/// Configuration for all six engines.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EngineConfig {
    pub cfd: liquidity_cfd::CfdConfig,
    pub gravity: gravity_whale::GravityConfig,
    pub hawkes: cascade_hawkes::HawkesConfig,
    pub neural: neural_panic::NeuralConfig,
    pub narrative: narrative_nlp::NarrativeConfig,
    pub bayesian: bayesian_risk::BayesianConfig,
    pub topology: topology::TopologyConfig,
}

/// A market event injected into the system.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MarketEvent {
    Tick { 
        symbol: String, 
        price: f64, 
        volume: f64, 
        timestamp_ns: u64,
        bid_depth: Vec<(f64, f64)>,
        ask_depth: Vec<(f64, f64)>,
    },
    Liquidation {
        symbol: String,
        side: OrderSide,
        size: f64,
        price: f64,
        timestamp_ns: u64,
    },
    NarrativePulse {
        source: String,
        sentiment_score: f64,
        entropy_bits: f64,
        viral_coefficient: f64,
    },
    WhaleMovement {
        wallet: String,
        symbol: String,
        delta: f64,
        timestamp_ns: u64,
    },
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum OrderSide { Buy, Sell }

#[derive(Debug, thiserror::Error)]
pub enum OmniversaError {
    #[error("Topology error: {0}")]
    Topology(String),
    #[error("Engine error: {0}")]
    Engine(String),
    #[error("Manifold error: {0}")]
    Manifold(String),
    #[error("Consensus failure: {0}")]
    Consensus(String),
}

/// The complete fragility snapshot — what the dashboard consumes.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FragilitySnapshot {
    pub timestamp_ns: u64,
    pub systemic_risk_field: RiskField,
    pub cascade_probability_tensor: Vec<f64>,
    pub volatility_weather_map: Vec<WeatherCell>,
    pub liquidity_collapse_radar: Vec<VacuumZone>,
    pub narrative_shock_index: f64,
    pub phase_space_position: ManifoldPoint,
    pub fragility_score: f64,
    pub engines_status: Vec<EngineStatus>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WeatherCell {
    pub x: f64, pub y: f64, pub z: f64,
    pub turbulence: f64,
    pub narrative_pressure: f64,
    pub liquidity_viscosity: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VacuumZone {
    pub price_level: f64,
    pub depth_deficit: f64,
    pub cavitation_risk: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EngineStatus {
    pub kind: String,
    pub health: f64,
    pub last_update_ns: u64,
    pub confidence: f64,
}
