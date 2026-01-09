//! Error types for the FPL optimizer

use thiserror::Error;

/// Result type alias using FplError
pub type Result<T> = std::result::Result<T, FplError>;

/// Main error type for the FPL optimizer
#[derive(Error, Debug)]
pub enum FplError {
    #[error("Data error: {0}")]
    Data(#[from] DataError),

    #[error("Solver error: {0}")]
    Solver(#[from] SolverError),

    #[error("Constraint error: {0}")]
    Constraint(#[from] ConstraintError),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
}

/// Errors related to data loading and parsing
#[derive(Error, Debug)]
pub enum DataError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("CSV parsing error: {0}")]
    CsvParse(String),

    #[error("Missing required column: {0}")]
    MissingColumn(String),

    #[error("Invalid data format: {0}")]
    InvalidFormat(String),

    #[error("API request failed: {0}")]
    ApiError(String),

    #[error("Player not found: {0}")]
    PlayerNotFound(u32),

    #[error("No players loaded")]
    NoPlayersLoaded,

    #[error("No projections loaded")]
    NoProjectionsLoaded,
}

/// Errors from the optimization solver
#[derive(Error, Debug)]
pub enum SolverError {
    #[error("Model is infeasible - no valid solution exists")]
    Infeasible,

    #[error("Solver timed out after {0} seconds")]
    Timeout(u64),

    #[error("Solver returned unbounded solution")]
    Unbounded,

    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    #[error("Failed to build model: {0}")]
    ModelBuildError(String),

    #[error("Solver internal error: {0}")]
    Internal(String),

    #[error("No solution found")]
    NoSolution,
}

/// Errors during constraint construction
#[derive(Error, Debug)]
pub enum ConstraintError {
    #[error("Invalid constraint: {0}")]
    Invalid(String),

    #[error("Conflicting constraints: {0}")]
    Conflict(String),

    #[error("Missing data for constraint: {0}")]
    MissingData(String),
}

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config file not found: {0}")]
    FileNotFound(String),

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Parse error: {0}")]
    Parse(String),
}
