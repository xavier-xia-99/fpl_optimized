//! FPL Optimizer - Fantasy Premier League Squad Optimization
//!
//! This crate provides Mixed Integer Linear Programming (MILP) based
//! optimization for Fantasy Premier League team selection.
//!
//! # Features
//!
//! - Multiple problem variants (Limited Squad, Bench Boost, Differential, etc.)
//! - Constraint-based modular architecture
//! - CBC solver backend via `good_lp`
//! - CSV and JSON output formats
//!
//! # Example
//!
//! ```rust,ignore
//! use fpl_optimizer::{
//!     problems::LimitedBestSquad,
//!     solver::CbcSolver,
//!     data::DataLoader,
//! };
//!
//! let data = DataLoader::from_csv("data/")?;
//! let problem = LimitedBestSquad::default();
//! let solver = CbcSolver::new();
//!
//! let result = problem.solve(&solver, &data)?;
//! println!("Best squad: {:?}", result);
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
pub use types::{Cost, Gameweek, Player, PlayerId, Position, TeamCode};
