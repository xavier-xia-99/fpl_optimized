//! Configuration management

use crate::error::{ConfigError, FplError, Result};
use crate::types::Cost;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// General settings
    #[serde(default)]
    pub general: GeneralConfig,

    /// Solver settings
    #[serde(default)]
    pub solver: SolverConfig,

    /// Constraint parameters
    #[serde(default)]
    pub constraints: ConstraintConfig,

    /// Bench weight settings
    #[serde(default)]
    pub bench: BenchConfig,

    /// Differential settings
    #[serde(default)]
    pub differential: DifferentialConfig,

    /// Iterative solving settings
    #[serde(default)]
    pub iterative: IterativeConfig,

    /// Output settings
    #[serde(default)]
    pub output: OutputConfig,
}

impl Config {
    /// Load configuration from file
    pub fn load(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path).map_err(|e| {
            FplError::Config(ConfigError::FileNotFound(format!("{}: {}", path.display(), e)))
        })?;

        toml::from_str(&contents)
            .map_err(|e| FplError::Config(ConfigError::Parse(e.to_string())))
    }

    /// Load from string
    pub fn from_str(s: &str) -> Result<Self> {
        toml::from_str(s).map_err(|e| FplError::Config(ConfigError::Parse(e.to_string())))
    }

    /// Get budget as Cost
    pub fn budget(&self) -> Cost {
        Cost(self.constraints.budget)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            solver: SolverConfig::default(),
            constraints: ConstraintConfig::default(),
            bench: BenchConfig::default(),
            differential: DifferentialConfig::default(),
            iterative: IterativeConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

/// General configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Season identifier
    #[serde(default = "default_season")]
    pub season: String,

    /// Data directory path
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            season: default_season(),
            data_dir: default_data_dir(),
        }
    }
}

fn default_season() -> String {
    "2025-26".to_string()
}

fn default_data_dir() -> String {
    "./data".to_string()
}

/// Solver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverConfig {
    /// Solver backend
    #[serde(default)]
    pub backend: SolverBackend,

    /// Time limit in seconds
    #[serde(default = "default_time_limit")]
    pub time_limit_seconds: u64,

    /// Number of threads
    #[serde(default = "default_threads")]
    pub threads: u8,

    /// MIP gap tolerance
    #[serde(default = "default_mip_gap")]
    pub mip_gap: f64,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            backend: SolverBackend::default(),
            time_limit_seconds: default_time_limit(),
            threads: default_threads(),
            mip_gap: default_mip_gap(),
        }
    }
}

fn default_time_limit() -> u64 {
    60
}

fn default_threads() -> u8 {
    4
}

fn default_mip_gap() -> f64 {
    0.0001
}

/// Supported solver backends
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SolverBackend {
    #[default]
    Cbc,
    Highs,
}

/// Constraint parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintConfig {
    /// Budget in tenths of millions
    #[serde(default = "default_budget")]
    pub budget: u16,

    /// Maximum players per team
    #[serde(default = "default_team_limit")]
    pub team_limit: u8,

    /// Lineup size
    #[serde(default = "default_lineup_size")]
    pub lineup_size: u8,

    /// Squad size
    #[serde(default = "default_squad_size")]
    pub squad_size: u8,
}

impl Default for ConstraintConfig {
    fn default() -> Self {
        Self {
            budget: default_budget(),
            team_limit: default_team_limit(),
            lineup_size: default_lineup_size(),
            squad_size: default_squad_size(),
        }
    }
}

fn default_budget() -> u16 {
    1000
}

fn default_team_limit() -> u8 {
    3
}

fn default_lineup_size() -> u8 {
    11
}

fn default_squad_size() -> u8 {
    15
}

/// Bench weight configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchConfig {
    /// Weight for bench players in objective
    #[serde(default = "default_bench_weight")]
    pub weight: f64,
}

impl Default for BenchConfig {
    fn default() -> Self {
        Self {
            weight: default_bench_weight(),
        }
    }
}

fn default_bench_weight() -> f64 {
    0.1
}

/// Differential configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferentialConfig {
    /// Maximum ownership percentage
    #[serde(default = "default_max_ownership")]
    pub max_ownership: f64,
}

impl Default for DifferentialConfig {
    fn default() -> Self {
        Self {
            max_ownership: default_max_ownership(),
        }
    }
}

fn default_max_ownership() -> f64 {
    5.0
}

/// Iterative solving configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterativeConfig {
    /// Number of iterations
    #[serde(default = "default_iterations")]
    pub iterations: usize,

    /// Squad overlap cutoff
    #[serde(default = "default_cutoff")]
    pub cutoff: usize,

    /// Random seed
    #[serde(default)]
    pub random_seed: Option<u64>,
}

impl Default for IterativeConfig {
    fn default() -> Self {
        Self {
            iterations: default_iterations(),
            cutoff: default_cutoff(),
            random_seed: Some(42),
        }
    }
}

fn default_iterations() -> usize {
    50
}

fn default_cutoff() -> usize {
    12
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output format
    #[serde(default)]
    pub format: OutputFormat,

    /// Include MPS export
    #[serde(default)]
    pub include_mps: bool,

    /// Output directory
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::default(),
            include_mps: false,
            output_dir: default_output_dir(),
        }
    }
}

fn default_output_dir() -> String {
    "./output".to_string()
}

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Csv,
    Json,
    Both,
}
