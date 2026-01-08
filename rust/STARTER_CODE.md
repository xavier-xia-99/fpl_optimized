# Starter Code Scaffolding

This document provides the initial code structure to bootstrap the Rust implementation. Copy these files to start development.

---

## File: `src/lib.rs`

```rust
//! FPL Optimizer - Fantasy Premier League Squad Optimization
//!
//! This crate provides Mixed Integer Linear Programming (MILP) based
//! optimization for Fantasy Premier League team selection.
//!
//! # Example
//!
//! ```rust
//! use fpl_optimizer::{
//!     problems::LimitedBestSquad,
//!     solver::CbcSolver,
//!     data::DataLoader,
//! };
//!
//! let data = DataLoader::from_csv("data/")?;
//! let solver = CbcSolver::new();
//! let problem = LimitedBestSquad::default();
//!
//! let result = problem.solve(&solver, &data)?;
//! println!("Best squad: {:?}", result.squad);
//! ```

pub mod config;
pub mod constraints;
pub mod data;
pub mod error;
pub mod model;
pub mod output;
pub mod problems;
pub mod solver;
pub mod types;

// Re-exports for convenience
pub use config::Config;
pub use error::{FplError, Result};
pub use types::{Cost, Gameweek, PlayerId, Player, Position, TeamCode};
```

---

## File: `src/types/mod.rs`

```rust
//! Domain types for FPL optimization

mod cost;
mod gameweek;
mod player;
mod position;
mod projection;
mod team;

pub use cost::Cost;
pub use gameweek::Gameweek;
pub use player::{Player, PlayerId};
pub use position::Position;
pub use projection::{PlayerGameweekProjection, ProjectionData};
pub use team::{Team, TeamCode};
```

---

## File: `src/types/position.rs`

```rust
//! FPL position types with constraints

use serde::{Deserialize, Serialize};
use std::fmt;

/// Fantasy Premier League positions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Position {
    Goalkeeper = 1,
    Defender = 2,
    Midfielder = 3,
    Forward = 4,
}

impl Position {
    /// All positions in order
    pub const ALL: [Position; 4] = [
        Position::Goalkeeper,
        Position::Defender,
        Position::Midfielder,
        Position::Forward,
    ];

    /// Iterator over all positions
    pub fn all() -> impl Iterator<Item = Position> {
        Self::ALL.iter().copied()
    }

    /// Number of this position required in squad (total = 15)
    pub const fn squad_select(&self) -> u8 {
        match self {
            Position::Goalkeeper => 2,
            Position::Defender => 5,
            Position::Midfielder => 5,
            Position::Forward => 3,
        }
    }

    /// Minimum starters for this position
    pub const fn min_play(&self) -> u8 {
        match self {
            Position::Goalkeeper => 1,
            Position::Defender => 3,
            Position::Midfielder => 2,
            Position::Forward => 1,
        }
    }

    /// Maximum starters for this position
    pub const fn max_play(&self) -> u8 {
        match self {
            Position::Goalkeeper => 1,
            Position::Defender => 5,
            Position::Midfielder => 5,
            Position::Forward => 3,
        }
    }

    /// Short name (e.g., "GK", "DEF")
    pub const fn short_name(&self) -> &'static str {
        match self {
            Position::Goalkeeper => "GK",
            Position::Defender => "DEF",
            Position::Midfielder => "MID",
            Position::Forward => "FWD",
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.short_name())
    }
}

impl TryFrom<u8> for Position {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Position::Goalkeeper),
            2 => Ok(Position::Defender),
            3 => Ok(Position::Midfielder),
            4 => Ok(Position::Forward),
            _ => Err("Invalid position ID (must be 1-4)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_squad_totals_15() {
        let total: u8 = Position::all().map(|p| p.squad_select()).sum();
        assert_eq!(total, 15);
    }

    #[test]
    fn test_min_play_totals_7() {
        let total: u8 = Position::all().map(|p| p.min_play()).sum();
        assert_eq!(total, 7);
    }

    #[test]
    fn test_max_play_totals_14() {
        let total: u8 = Position::all().map(|p| p.max_play()).sum();
        assert_eq!(total, 14);
    }

    #[test]
    fn test_try_from_valid() {
        assert_eq!(Position::try_from(1), Ok(Position::Goalkeeper));
        assert_eq!(Position::try_from(4), Ok(Position::Forward));
    }

    #[test]
    fn test_try_from_invalid() {
        assert!(Position::try_from(0).is_err());
        assert!(Position::try_from(5).is_err());
    }
}
```

---

## File: `src/types/cost.rs`

```rust
//! Player cost type

use serde::{Deserialize, Serialize};
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, Sub};

/// Player cost in tenths of millions (e.g., 100 = £10.0M)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Cost(pub u16);

impl Cost {
    /// Default FPL budget: £100.0M = 1000 tenths
    pub const BUDGET_DEFAULT: Cost = Cost(1000);

    /// Minimum player cost: £4.0M
    pub const MIN: Cost = Cost(40);

    /// Maximum player cost: ~£15.0M
    pub const MAX: Cost = Cost(150);

    /// Create from millions (e.g., 10.5 -> Cost(105))
    pub fn from_millions(millions: f64) -> Self {
        Cost((millions * 10.0).round() as u16)
    }

    /// Convert to millions (e.g., Cost(105) -> 10.5)
    pub fn to_millions(&self) -> f64 {
        self.0 as f64 / 10.0
    }

    /// Raw value in tenths of millions
    pub const fn raw(&self) -> u16 {
        self.0
    }
}

impl fmt::Display for Cost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "£{:.1}M", self.to_millions())
    }
}

impl Add for Cost {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Cost(self.0.saturating_add(rhs.0))
    }
}

impl Sub for Cost {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Cost(self.0.saturating_sub(rhs.0))
    }
}

impl Sum for Cost {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Cost(0), |acc, c| acc + c)
    }
}

impl<'a> Sum<&'a Cost> for Cost {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Cost(0), |acc, c| acc + *c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_millions() {
        assert_eq!(Cost::from_millions(10.0), Cost(100));
        assert_eq!(Cost::from_millions(10.5), Cost(105));
        assert_eq!(Cost::from_millions(4.5), Cost(45));
    }

    #[test]
    fn test_to_millions() {
        assert_eq!(Cost(100).to_millions(), 10.0);
        assert_eq!(Cost(105).to_millions(), 10.5);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Cost(100)), "£10.0M");
        assert_eq!(format!("{}", Cost(127)), "£12.7M");
    }

    #[test]
    fn test_sum() {
        let costs = vec![Cost(50), Cost(60), Cost(45)];
        let total: Cost = costs.iter().sum();
        assert_eq!(total, Cost(155));
    }
}
```

---

## File: `src/types/player.rs`

```rust
//! Player entity types

use crate::types::{Cost, Position, TeamCode};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique player identifier from FPL API
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub u32);

impl PlayerId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A Fantasy Premier League player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// Unique identifier
    pub id: PlayerId,

    /// Display name (e.g., "Salah")
    pub web_name: String,

    /// Team code
    pub team_code: TeamCode,

    /// Playing position
    pub position: Position,

    /// Current price
    pub cost: Cost,

    /// Ownership percentage (0.0 - 100.0)
    pub ownership_percent: f64,

    // Optional statistics
    pub form: Option<f64>,
    pub ict_index: Option<f64>,
    pub ep_this: Option<f64>,
    pub points_per_game: Option<f64>,
}

impl Player {
    /// Create a minimal player for testing
    #[cfg(test)]
    pub fn test(id: u32, position: Position, cost: u16, team: u8) -> Self {
        Self {
            id: PlayerId(id),
            web_name: format!("Player{}", id),
            team_code: TeamCode(team),
            position,
            cost: Cost(cost),
            ownership_percent: 5.0,
            form: None,
            ict_index: None,
            ep_this: None,
            points_per_game: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        let player = Player::test(1, Position::Midfielder, 127, 14);
        assert_eq!(player.id, PlayerId(1));
        assert_eq!(player.position, Position::Midfielder);
        assert_eq!(player.cost, Cost(127));
    }
}
```

---

## File: `src/error.rs`

```rust
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
```

---

## File: `src/constraints/mod.rs`

```rust
//! Optimization constraints for FPL problems

mod bench;
mod budget;
mod captain;
mod differential;
mod iterative;
mod lineup_size;
mod lineup_squad;
mod position;
mod squad_size;
mod team_limit;

pub use bench::BenchConstraint;
pub use budget::BudgetConstraint;
pub use captain::{CaptainMustPlayConstraint, SingleCaptainConstraint};
pub use differential::DifferentialConstraint;
pub use iterative::IterativeCutoffConstraint;
pub use lineup_size::LineupSizeConstraint;
pub use lineup_squad::LineupSquadConstraint;
pub use position::{LineupPositionConstraint, SquadPositionConstraint};
pub use squad_size::SquadSizeConstraint;
pub use team_limit::TeamLimitConstraint;

use crate::error::ConstraintError;
use crate::model::ModelBuilder;
use crate::types::Player;

/// Data required to build constraints
pub struct ProblemData<'a> {
    pub players: &'a [Player],
    // Add more fields as needed
}

/// Trait for optimization constraints
pub trait Constraint {
    /// Unique name for this constraint
    fn name(&self) -> &'static str;

    /// Apply this constraint to the model builder
    fn apply(
        &self,
        builder: &mut ModelBuilder,
        data: &ProblemData,
    ) -> Result<(), ConstraintError>;

    /// Whether this constraint is required (hard) or optional (soft)
    fn is_required(&self) -> bool {
        true
    }
}
```

---

## File: `src/constraints/squad_size.rs`

```rust
//! Squad size constraint: exactly 15 players

use super::{Constraint, ProblemData};
use crate::error::ConstraintError;
use crate::model::ModelBuilder;

/// Constraint: Squad must contain exactly N players (default 15)
#[derive(Debug, Clone)]
pub struct SquadSizeConstraint {
    pub size: u8,
}

impl Default for SquadSizeConstraint {
    fn default() -> Self {
        Self { size: 15 }
    }
}

impl Constraint for SquadSizeConstraint {
    fn name(&self) -> &'static str {
        "squad_limit"
    }

    fn apply(
        &self,
        builder: &mut ModelBuilder,
        data: &ProblemData,
    ) -> Result<(), ConstraintError> {
        // Σ z[i] = size
        builder.add_squad_sum_constraint(data.players, self.size, self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_size() {
        let c = SquadSizeConstraint::default();
        assert_eq!(c.size, 15);
    }
}
```

---

## File: `src/model/mod.rs`

```rust
//! MILP model construction

mod builder;
mod objective;
mod variables;

pub use builder::ModelBuilder;
pub use objective::Objective;
pub use variables::Variable;
```

---

## File: `src/model/builder.rs`

```rust
//! Model builder for constructing optimization models

use crate::error::ConstraintError;
use crate::types::{Player, PlayerId};
use std::collections::HashMap;

/// Builder for constructing MILP models
pub struct ModelBuilder {
    lineup_vars: HashMap<PlayerId, usize>,
    captain_vars: HashMap<PlayerId, usize>,
    squad_vars: HashMap<PlayerId, usize>,
    // TODO: Add actual model representation
}

impl ModelBuilder {
    pub fn new() -> Self {
        Self {
            lineup_vars: HashMap::new(),
            captain_vars: HashMap::new(),
            squad_vars: HashMap::new(),
        }
    }

    /// Initialize variables for all players
    pub fn init_variables(&mut self, players: &[Player]) {
        for (idx, player) in players.iter().enumerate() {
            self.lineup_vars.insert(player.id, idx * 3);
            self.captain_vars.insert(player.id, idx * 3 + 1);
            self.squad_vars.insert(player.id, idx * 3 + 2);
        }
    }

    /// Add constraint: sum of squad variables = target
    pub fn add_squad_sum_constraint(
        &mut self,
        players: &[Player],
        target: u8,
        name: &str,
    ) -> Result<(), ConstraintError> {
        // TODO: Implement actual constraint addition
        let _ = (players, target, name);
        Ok(())
    }

    /// Build the final model
    pub fn build(self) -> Model {
        Model {
            // TODO: Build actual model
        }
    }
}

impl Default for ModelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Completed optimization model ready for solving
pub struct Model {
    // TODO: Add model fields
}
```

---

## File: `src/solver/mod.rs`

```rust
//! Solver backends for MILP optimization

mod cbc;
mod solution;
mod traits;

pub use cbc::CbcSolver;
pub use solution::Solution;
pub use traits::Solver;
```

---

## File: `src/solver/traits.rs`

```rust
//! Solver trait definition

use crate::error::SolverError;
use crate::model::Model;
use crate::solver::Solution;
use std::path::Path;

/// Abstract interface for MILP solvers
pub trait Solver {
    /// Solve the model and return a solution
    fn solve(&self, model: &Model) -> Result<Solution, SolverError>;

    /// Export model to MPS format
    fn export_mps(&self, model: &Model, path: &Path) -> Result<(), SolverError>;

    /// Set solving time limit in seconds
    fn set_time_limit(&mut self, seconds: u64);

    /// Get solver name for logging
    fn name(&self) -> &str;
}
```

---

## File: `src/main.rs`

```rust
//! FPL Optimizer CLI

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "fpl")]
#[command(author, version, about = "FPL Squad Optimization using MILP")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Solve an optimization problem
    Solve {
        /// Gameweek to optimize for
        #[arg(short, long)]
        gameweek: Option<u8>,

        /// Budget in millions (e.g., 100.0)
        #[arg(short, long)]
        budget: Option<f64>,
    },

    /// Generate multiple diverse squads
    Iterative {
        /// Number of squads to generate
        #[arg(short, long, default_value = "50")]
        count: usize,

        /// Gameweek to optimize for
        #[arg(short, long)]
        gameweek: Option<u8>,
    },

    /// Update data from FPL API
    Update,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    match cli.command {
        Commands::Solve { gameweek, budget } => {
            tracing::info!("Solving for GW {:?}, budget {:?}", gameweek, budget);
            // TODO: Implement solving
            println!("Solving not yet implemented");
        }
        Commands::Iterative { count, gameweek } => {
            tracing::info!(
                "Generating {} squads for GW {:?}",
                count,
                gameweek
            );
            // TODO: Implement iterative solving
            println!("Iterative solving not yet implemented");
        }
        Commands::Update => {
            tracing::info!("Updating data from FPL API");
            // TODO: Implement data update
            println!("Data update not yet implemented");
        }
    }

    Ok(())
}
```

---

## Next Steps

After copying these files:

1. Run `cargo build` to verify compilation
2. Run `cargo test` to run unit tests
3. Implement remaining modules following the patterns shown
4. Refer to [TODO_CHECKLIST.md](TODO_CHECKLIST.md) for task tracking

---

*Last Updated: 2026-01-08*
