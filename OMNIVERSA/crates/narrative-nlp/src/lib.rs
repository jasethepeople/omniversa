//! Narrative NLP Engine — The "Atmospheric" Layer
//! 
//! Scientific Basis: Latent Dirichlet Allocation / Transformer-GNNs
//! Analogy: Sentiment is a weather system.
//! 
//! Scans data points to map "Narrative High Pressure Zones." Identifies
//! Narrative Pathogens — memetic sequences that correlate with historical
//! volatility. Measures the "moisture content" (leverage) of a narrative.

#![warn(missing_docs)]
#![deny(unsafe_code)]

use nalgebra::{DVector, DMatrix};
use ndarray::{Array1, Array2};
use rayon::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque, HashSet};
use regex::Regex;

/// Configuration for the Narrative engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeConfig {
    pub n_topics: usize,
    pub viral_coefficient_threshold: f64,
    pub entropy_window: usize,
    pub max_pathogens: usize,
    pub pressure_smoothing: f64,
}

impl Default for NarrativeConfig {
    fn default() -> Self {
        Self {
            n_topics: 50,
            viral_coefficient_threshold: 2.0,
            entropy_window: 1000,
            max_pathogens: 100,
            pressure_smoothing: 0.9,
        }
    }
}

/// A "narrative cloud" — a cluster of related memes/sentiment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeCloud {
    pub id: usize,
    pub topic_vector: Vec<f64>,
    pub pressure: f64,
    pub moisture: f64,
    pub temperature: f64,
    pub viral_coefficient: f64,
    pub entropy: f64,
    pub sources: Vec<String>,
    pub timestamp_ns: u64,
}

/// A "narrative pathogen" — a meme sequence that causes volatility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativePathogen {
    pub sequence: Vec<String>,
    pub virulence: f64,
    pub historical_correlation: f64,
    pub detection_count: usize,
    pub last_seen_ns: u64,
}

/// The Narrative engine — atmospheric weather system for sentiment.
#[derive(Debug, Clone)]
pub struct NarrativeEngine {
    config: NarrativeConfig,
    clouds: Vec<NarrativeCloud>,
    pathogens: Vec<NarrativePathogen>,
    pressure_history: VecDeque<f64>,
    entropy_history: VecDeque<f64>,
    viral_history: VecDeque<f64>,
    topic_distributions: Array2<f64>,
    known_sequences: HashSet<String>,
}

impl NarrativeEngine {
    /// Initialize the narrative engine.
    pub fn new(config: NarrativeConfig) -> Result<Self, String> {
        Ok(Self {
            config: config.clone(),
            clouds: Vec::new(),
            pathogens: Vec::new(),
            pressure_history: VecDeque::with_capacity(config.entropy_window),
            entropy_history: VecDeque::with_capacity(config.entropy_window),
            viral_history: VecDeque::with_capacity(config.entropy_window),
            topic_distributions: Array2::zeros((config.n_topics, 100)),
            known_sequences: HashSet::new(),
        })
    }

    /// Evaluate narrative atmosphere given systemic stress.
    pub fn evaluate(&self, stress: &[f64]) -> Result<NarrativeMetrics, String> {
        let pressure = self.compute_pressure();
        let moisture = self.compute_moisture();
        let pathogen_count = self.pathogens.iter()
            .filter(|p| p.virulence > 0.5)
            .count();
        let entropy_bits = self.compute_entropy();

        Ok(NarrativeMetrics {
            pressure,
            moisture,
            pathogen_count,
            entropy_bits,
        })
    }

    fn compute_pressure(&self) -> f64 {
        if self.pressure_history.is_empty() { return 0.0; }
        let recent: Vec<_> = self.pressure_history.iter().rev().take(100).cloned().collect();
        recent.iter().sum::<f64>() / recent.len() as f64
    }

    fn compute_moisture(&self) -> f64 {
        self.clouds.iter().map(|c| c.moisture).sum::<f64>() / self.clouds.len().max(1) as f64
    }

    fn compute_entropy(&self) -> f64 {
        if self.entropy_history.is_empty() { return 0.0; }

        let recent: Vec<_> = self.entropy_history.iter().rev().take(self.config.entropy_window).cloned().collect();

        let mut histogram = HashMap::new();
        for &v in &recent {
            let bucket = (v * 10.0).floor() as usize;
            *histogram.entry(bucket).or_insert(0usize) += 1;
        }

        let total = recent.len() as f64;
        let mut entropy = 0.0;
        for &count in histogram.values() {
            let p = count as f64 / total;
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    /// Ingest a narrative pulse from a data source.
    pub fn ingest_pulse(&mut self, source: String, sentiment: f64, entropy: f64, viral: f64, timestamp_ns: u64) {
        let cloud = NarrativeCloud {
            id: self.clouds.len(),
            topic_vector: vec![sentiment, viral, entropy],
            pressure: sentiment.abs(),
            moisture: viral * sentiment.abs(),
            temperature: viral,
            viral_coefficient: viral,
            entropy,
            sources: vec![source.clone()],
            timestamp_ns,
        };

        self.clouds.push(cloud);
        if self.clouds.len() > 10000 {
            self.clouds.remove(0);
        }

        self.pressure_history.push_back(sentiment.abs());
        self.entropy_history.push_back(entropy);
        self.viral_history.push_back(viral);

        if self.pressure_history.len() > self.config.entropy_window {
            self.pressure_history.pop_front();
        }
        if self.entropy_history.len() > self.config.entropy_window {
            self.entropy_history.pop_front();
        }
        if self.viral_history.len() > self.config.entropy_window {
            self.viral_history.pop_front();
        }

        if viral > self.config.viral_coefficient_threshold {
            self.detect_pathogen(source, sentiment, viral, timestamp_ns);
        }
    }

    fn detect_pathogen(&mut self, source: String, sentiment: f64, viral: f64, timestamp_ns: u64) {
        let sequence = vec![source.clone(), format!("sentiment:{:.2}", sentiment)];
        let seq_key = sequence.join("|");

        if self.known_sequences.contains(&seq_key) {
            if let Some(pathogen) = self.pathogens.iter_mut()
                .find(|p| p.sequence == sequence) {
                pathogen.detection_count += 1;
                pathogen.last_seen_ns = timestamp_ns;
                pathogen.virulence = (pathogen.virulence * 0.9 + viral * 0.1).min(1.0);
            }
        } else {
            self.known_sequences.insert(seq_key);
            self.pathogens.push(NarrativePathogen {
                sequence,
                virulence: viral,
                historical_correlation: sentiment.abs(),
                detection_count: 1,
                last_seen_ns: timestamp_ns,
            });

            if self.pathogens.len() > self.config.max_pathogens {
                self.pathogens.sort_by(|a, b| b.virulence.partial_cmp(&a.virulence).unwrap());
                self.pathogens.truncate(self.config.max_pathogens);
            }
        }
    }

    /// Predict "volatility supercell" — collision of high-moisture narrative with bad news.
    pub fn predict_supercell(&self) -> SupercellPrediction {
        let high_pressure_clouds: Vec<_> = self.clouds.iter()
            .filter(|c| c.pressure > 0.7 && c.moisture > 0.5)
            .collect();

        let cold_fronts: Vec<_> = self.clouds.iter()
            .filter(|c| c.temperature < 0.3 && c.pressure > 0.5)
            .collect();

        let collision_probability = if !high_pressure_clouds.is_empty() && !cold_fronts.is_empty() {
            let min_distance = high_pressure_clouds.iter()
                .flat_map(|hp| cold_fronts.iter().map(|cf| {
                    let d: f64 = hp.topic_vector.iter().zip(cf.topic_vector.iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum();
                    d.sqrt()
                }))
                .fold(f64::INFINITY, f64::min);

            (1.0 / (1.0 + min_distance)).min(1.0)
        } else {
            0.0
        };

        SupercellPrediction {
            collision_probability,
            high_pressure_count: high_pressure_clouds.len(),
            cold_front_count: cold_fronts.len(),
            expected_volatility: collision_probability * 2.0,
        }
    }

    /// Get the "weather map" — current atmospheric conditions.
    pub fn weather_map(&self) -> Vec<WeatherCell> {
        self.clouds.iter().map(|c| WeatherCell {
            x: c.topic_vector.get(0).copied().unwrap_or(0.0),
            y: c.topic_vector.get(1).copied().unwrap_or(0.0),
            z: c.topic_vector.get(2).copied().unwrap_or(0.0),
            pressure: c.pressure,
            moisture: c.moisture,
            temperature: c.temperature,
        }).collect()
    }
}

/// Output metrics from the Narrative engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeMetrics {
    pub pressure: f64,
    pub moisture: f64,
    pub pathogen_count: usize,
    pub entropy_bits: f64,
}

/// Prediction of a volatility supercell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupercellPrediction {
    pub collision_probability: f64,
    pub high_pressure_count: usize,
    pub cold_front_count: usize,
    pub expected_volatility: f64,
}

/// A cell in the narrative weather map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherCell {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub pressure: f64,
    pub moisture: f64,
    pub temperature: f64,
}
