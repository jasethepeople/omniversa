//! Phase Space Manifold — The 4D+ visualization space of market thermodynamics.
//! 
//! Instead of price charts, we operate on a high-dimensional phase space where
//! each axis represents a thermodynamic variable of the market.

use nalgebra::{DVector, DMatrix, SymmetricEigen};
use ndarray::{Array1, Array2, Axis};
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

/// A point in the 6-dimensional phase space.
/// Dimensions: [Liquidity_Viscosity, Whale_Gravity, Cascade_Stress, 
///               Neural_Voltage, Narrative_Pressure, Bayesian_Entropy]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ManifoldPoint {
    pub liquidity_viscosity: f64,
    pub whale_gravity: f64,
    pub cascade_stress: f64,
    pub neural_voltage: f64,
    pub narrative_pressure: f64,
    pub bayesian_entropy: f64,
    pub timestamp_ns: u64,
}

impl ManifoldPoint {
    pub fn new(timestamp_ns: u64) -> Self {
        Self {
            liquidity_viscosity: 0.0,
            whale_gravity: 0.0,
            cascade_stress: 0.0,
            neural_voltage: 0.0,
            narrative_pressure: 0.0,
            bayesian_entropy: 0.0,
            timestamp_ns,
        }
    }

    pub fn as_vector(&self) -> DVector<f64> {
        DVector::from_vec(vec![
            self.liquidity_viscosity,
            self.whale_gravity,
            self.cascade_stress,
            self.neural_voltage,
            self.narrative_pressure,
            self.bayesian_entropy,
        ])
    }

    /// Compute the Lyapunov-like divergence from another point.
    /// High divergence = the system is becoming unpredictable (chaos).
    pub fn divergence(&self, other: &ManifoldPoint) -> f64 {
        let dv = self.as_vector() - other.as_vector();
        dv.norm_squared().sqrt()
    }

    /// Compute the "temperature" of this point — total kinetic energy of the market.
    pub fn temperature(&self) -> f64 {
        self.as_vector().norm_squared()
    }
}

/// The Phase Space Manifold maintains a rolling window of points and computes
/// topological properties: attractors, repellors, and bifurcation boundaries.
pub struct PhaseSpaceManifold {
    dimensions: usize,
    history: VecDeque<ManifoldPoint>,
    max_history: usize,
    attractors: Vec<ManifoldPoint>,
    repellors: Vec<ManifoldPoint>,
    covariance: Array2<f64>,
}

impl PhaseSpaceManifold {
    pub fn new(dimensions: usize) -> Self {
        Self {
            dimensions,
            history: VecDeque::with_capacity(10000),
            max_history: 10000,
            attractors: Vec::new(),
            repellors: Vec::new(),
            covariance: Array2::zeros((dimensions, dimensions)),
        }
    }

    pub fn ingest(&mut self, point: ManifoldPoint) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(point);
        self.update_covariance();
        self.detect_attractors();
    }

    fn update_covariance(&mut self) {
        if self.history.len() < 2 { return; }

        let n = self.history.len();
        let mut mean = Array1::zeros(self.dimensions);

        for p in &self.history {
            mean[0] += p.liquidity_viscosity;
            mean[1] += p.whale_gravity;
            mean[2] += p.cascade_stress;
            mean[3] += p.neural_voltage;
            mean[4] += p.narrative_pressure;
            mean[5] += p.bayesian_entropy;
        }
        mean /= n as f64;

        let mut cov = Array2::zeros((self.dimensions, self.dimensions));
        for p in &self.history {
            let v = Array1::from_vec(vec![
                p.liquidity_viscosity - mean[0],
                p.whale_gravity - mean[1],
                p.cascade_stress - mean[2],
                p.neural_voltage - mean[3],
                p.narrative_pressure - mean[4],
                p.bayesian_entropy - mean[5],
            ]);
            cov += &v.view().insert_axis(Axis(1)).dot(&v.view().insert_axis(Axis(0)));
        }
        cov /= (n - 1) as f64;
        self.covariance = cov;
    }

    fn detect_attractors(&mut self) {
        // Simplified: points where the gradient of the flow is negative
        // (the system converges toward them)
        if self.history.len() < 10 { return; }

        self.attractors.clear();
        let recent: Vec<_> = self.history.iter().rev().take(100).cloned().collect();

        for window in recent.windows(3) {
            let d1 = window[1].divergence(&window[0]);
            let d2 = window[2].divergence(&window[1]);
            if d1 > d2 && d2 < 0.001 {
                self.attractors.push(window[2]);
            }
        }
    }

    /// Compute the principal components — the dominant modes of market motion.
    pub fn principal_components(&self) -> Vec<(f64, Vec<f64>)> {
        // Eigen-decomposition of covariance matrix
        let mat = DMatrix::from_row_slice(
            self.dimensions,
            self.dimensions,
            &self.covariance.as_slice().unwrap(),
        );

        let eigen = SymmetricEigen::new(mat);
        let mut pairs: Vec<_> = eigen.eigenvalues.iter()
            .zip(eigen.eigenvectors.column_iter())
            .map(|(val, vec)| (*val, vec.iter().cloned().collect()))
            .collect();

        pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        pairs
    }

    /// Predict the next manifold point using linear extrapolation of principal modes.
    pub fn extrapolate(&self, horizon_ns: u64) -> ManifoldPoint {
        if self.history.len() < 2 {
            return ManifoldPoint::new(horizon_ns);
        }

        let last = self.history.back().unwrap();
        let prev = self.history.iter().rev().nth(1).unwrap();
        let dt = last.timestamp_ns - prev.timestamp_ns;

        let velocity = last.as_vector() - prev.as_vector();
        let predicted = last.as_vector() + velocity * (horizon_ns as f64 / dt.max(1) as f64);

        ManifoldPoint {
            liquidity_viscosity: predicted[0],
            whale_gravity: predicted[1],
            cascade_stress: predicted[2],
            neural_voltage: predicted[3],
            narrative_pressure: predicted[4],
            bayesian_entropy: predicted[5],
            timestamp_ns: last.timestamp_ns + horizon_ns,
        }
    }

    /// Compute the "fractal dimension" of the attractor set.
    /// Low dimension = predictable. High dimension = chaos.
    pub fn correlation_dimension(&self) -> f64 {
        if self.history.len() < 100 { return 6.0; }

        let points: Vec<_> = self.history.iter().collect();
        let mut correlations = Vec::new();

        for r in [0.01, 0.05, 0.1, 0.2, 0.5] {
            let mut count = 0usize;
            for i in 0..points.len() {
                for j in (i+1)..points.len() {
                    if points[i].divergence(points[j]) < r {
                        count += 1;
                    }
                }
            }
            let c = count as f64 / (points.len() * (points.len() - 1) / 2) as f64;
            correlations.push((r.ln(), c.ln()));
        }

        // Linear regression on log-log plot
        let n = correlations.len() as f64;
        let sum_x: f64 = correlations.iter().map(|(x, _)| x).sum();
        let sum_y: f64 = correlations.iter().map(|(_, y)| y).sum();
        let sum_xy: f64 = correlations.iter().map(|(x, y)| x * y).sum();
        let sum_x2: f64 = correlations.iter().map(|(x, _)| x * x).sum();

        (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)
    }
}

/// A 3D heatmap field showing where structural stress accumulates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskField {
    pub grid: Vec<Vec<Vec<f64>>>,
    pub resolution: (usize, usize, usize),
    pub bounds: ((f64, f64), (f64, f64), (f64, f64)),
}

impl RiskField {
    pub fn new(res: (usize, usize, usize), bounds: ((f64, f64), (f64, f64), (f64, f64))) -> Self {
        Self {
            grid: vec![vec![vec![0.0; res.2]; res.1]; res.0],
            resolution: res,
            bounds,
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, value: f64) {
        self.grid[x][y][z] = value;
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> f64 {
        self.grid[x][y][z]
    }

    pub fn max_stress_location(&self) -> (usize, usize, usize) {
        let mut max_val = 0.0;
        let mut loc = (0, 0, 0);
        for x in 0..self.resolution.0 {
            for y in 0..self.resolution.1 {
                for z in 0..self.resolution.2 {
                    if self.grid[x][y][z] > max_val {
                        max_val = self.grid[x][y][z];
                        loc = (x, y, z);
                    }
                }
            }
        }
        loc
    }
}
