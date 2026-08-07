//! Telemetry — Distributed tracing and metrics for the sovereign engine.

use std::sync::atomic::{AtomicU64, Ordering};
use tokio::time::{interval, Duration};
use tracing::{info, debug};

pub struct TelemetryCollector {
    tick_count: AtomicU64,
    event_count: AtomicU64,
    engine_errors: AtomicU64,
    last_tick_latency_ns: AtomicU64,
}

impl TelemetryCollector {
    pub fn new() -> Self {
        Self {
            tick_count: AtomicU64::new(0),
            event_count: AtomicU64::new(0),
            engine_errors: AtomicU64::new(0),
            last_tick_latency_ns: AtomicU64::new(0),
        }
    }

    pub async fn record_tick(&self) {
        let count = self.tick_count.fetch_add(1, Ordering::Relaxed) + 1;
        if count % 100 == 0 {
            info!("Sovereign tick #{} completed", count);
        }
    }

    pub fn record_event(&self) {
        self.event_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.engine_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn metrics(&self) -> TelemetryMetrics {
        TelemetryMetrics {
            tick_count: self.tick_count.load(Ordering::Relaxed),
            event_count: self.event_count.load(Ordering::Relaxed),
            engine_errors: self.engine_errors.load(Ordering::Relaxed),
            last_tick_latency_ns: self.last_tick_latency_ns.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TelemetryMetrics {
    pub tick_count: u64,
    pub event_count: u64,
    pub engine_errors: u64,
    pub last_tick_latency_ns: u64,
}
