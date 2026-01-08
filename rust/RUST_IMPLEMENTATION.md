# FPL Optimization Solver - Rust Implementation Guide

## Executive Summary

This document provides a comprehensive implementation guide for porting the Fantasy Premier League (FPL) optimization solver from Python/sasoptpy to Rust. The solver uses **Mixed Integer Linear Programming (MILP)** to generate optimal squad selections and lineups.

**Target Audience:** Rust developers with LP/MILP background  
**Estimated Implementation Time:** 4-6 weeks  
**Complexity Rating:** High

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Module Structure](#module-structure)
3. [Core Data Types](#core-data-types)
4. [Solver Backend Selection](#solver-backend-selection)
5. [Constraint Modeling](#constraint-modeling)
6. [Problem Implementations](#problem-implementations)
7. [API Design](#api-design)
8. [Configuration System](#configuration-system)
9. [Testing Strategy](#testing-strategy)
10. [Performance Considerations](#performance-considerations)
11. [Implementation Roadmap](#implementation-roadmap)
12. [Appendix: Mathematical Formulation](#appendix-mathematical-formulation)

---

## Architecture Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           FPL Optimizer (Rust)                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────────────┐  │
│  │   Data       │───▶│   Model      │───▶│   Solver Backend         │  │
│  │   Ingestion  │    │   Builder    │    │   (CBC/HiGHS/Gurobi)     │  │
│  └──────────────┘    └──────────────┘    └──────────────────────────┘  │
│         │                   │                        │                  │
│         ▼                   ▼                        ▼                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────────────┐  │
│  │   Types      │    │  Constraint  │    │   Solution Parser        │  │
│  │   (Domain)   │    │  Registry    │    │   & Output               │  │
│  └──────────────┘    └──────────────┘    └──────────────────────────┘  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Design Principles

1. **Type Safety First** - Leverage Rust's type system to prevent constraint violations at compile time where possible
2. **Zero-Cost Abstractions** - Use traits for solver backends without runtime overhead
3. **Modularity** - Each problem variant is a composable set of constraints
4. **Parallelism Ready** - Design for concurrent solving (iterative solutions)
5. **Solver Agnostic** - Abstract solver interface for multiple backends

---

## Module Structure

```
fpl_optimizer/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Library root, re-exports
│   ├── main.rs                   # CLI entry point
│   │
│   ├── types/                    # Domain types
│   │   ├── mod.rs
│   │   ├── player.rs             # Player entity
│   │   ├── team.rs               # Team entity  
│   │   ├── position.rs           # Position enum (GK, DEF, MID, FWD)
│   │   ├── gameweek.rs           # Gameweek wrapper
│   │   ├── cost.rs               # Cost newtype (tenths of millions)
│   │   └── projection.rs         # Expected points data
│   │
│   ├── data/                     # Data loading & transformation
│   │   ├── mod.rs
│   │   ├── fpl_api.rs            # FPL API client (async)
│   │   ├── csv_loader.rs         # CSV projection loader
│   │   ├── schema.rs             # Data schemas
│   │   └── intermediate.rs       # element_gameweek generation
│   │
│   ├── model/                    # MILP model construction
│   │   ├── mod.rs
│   │   ├── variables.rs          # Variable definitions
│   │   ├── objective.rs          # Objective function builders
│   │   ├── constraint.rs         # Constraint trait & registry
│   │   └── builder.rs            # Model builder pattern
│   │
│   ├── constraints/              # Individual constraint implementations
│   │   ├── mod.rs
│   │   ├── squad_size.rs         # Σz[i] = 15
│   │   ├── lineup_size.rs        # Σx[i] = 11
│   │   ├── budget.rs             # Σz[i]×cost[i] ≤ budget
│   │   ├── team_limit.rs         # Max 3 per team
│   │   ├── position.rs           # Position min/max
│   │   ├── captain.rs            # Single captain
│   │   ├── lineup_squad.rs       # x[i] ≤ z[i]
│   │   ├── differential.rs       # Ownership threshold
│   │   ├── bench.rs              # Bench ordering
│   │   └── iterative.rs          # Squad cutoff for diversity
│   │
│   ├── problems/                 # Problem variants
│   │   ├── mod.rs
│   │   ├── traits.rs             # Problem trait definition
│   │   ├── no_limit_best_11.rs
│   │   ├── limited_best_squad.rs
│   │   ├── weighted_bench.rs
│   │   ├── bench_boost.rs
│   │   ├── differential.rs
│   │   ├── set_and_forget.rs
│   │   └── iterative.rs
│   │
│   ├── solver/                   # Solver backend abstraction
│   │   ├── mod.rs
│   │   ├── traits.rs             # Solver trait
│   │   ├── cbc.rs                # CBC backend (via coin_cbc)
│   │   ├── highs.rs              # HiGHS backend (via highs)
│   │   ├── solution.rs           # Solution representation
│   │   └── mps.rs                # MPS export functionality
│   │
│   ├── output/                   # Results formatting
│   │   ├── mod.rs
│   │   ├── csv.rs                # CSV output
│   │   ├── json.rs               # JSON output (iterative)
│   │   └── display.rs            # Terminal display
│   │
│   ├── config/                   # Configuration management
│   │   ├── mod.rs
│   │   ├── params.rs             # Tunable parameters
│   │   └── static_values.rs      # Static config loader
│   │
│   └── error.rs                  # Error types
│
├── benches/                      # Benchmarks
│   └── solver_benchmark.rs
│
└── tests/                        # Integration tests
    ├── problem_tests.rs
    └── constraint_tests.rs
```

---

## Core Data Types

### Player Entity

```rust
// src/types/player.rs

use crate::types::{Position, TeamCode, Cost, PlayerId};

/// Unique player identifier from FPL API
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u32);

/// Represents a player with all relevant attributes
#[derive(Debug, Clone)]
pub struct Player {
    pub id: PlayerId,
    pub web_name: String,
    pub team_code: TeamCode,
    pub position: Position,
    pub cost: Cost,
    pub ownership_percent: f64,
    
    // Optional stats (may not always be available)
    pub form: Option<f64>,
    pub ict_index: Option<f64>,
    pub ep_this: Option<f64>,
    pub points_per_game: Option<f64>,
}
```

### Position Enum

```rust
// src/types/position.rs

/// FPL positions with their constraints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Position {
    Goalkeeper = 1,
    Defender = 2,
    Midfielder = 3,
    Forward = 4,
}

impl Position {
    /// Number of this position required in squad
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
}
```

### Cost Newtype

```rust
// src/types/cost.rs

/// Player cost in tenths of millions (e.g., 100 = £10.0M)
/// Using newtype pattern for type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cost(pub u16);

impl Cost {
    pub const BUDGET_DEFAULT: Cost = Cost(1000); // £100.0M
    
    pub fn from_millions(millions: f64) -> Self {
        Cost((millions * 10.0) as u16)
    }
    
    pub fn to_millions(&self) -> f64 {
        self.0 as f64 / 10.0
    }
}

impl std::ops::Add for Cost {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Cost(self.0 + rhs.0)
    }
}
```

### Player Gameweek Projection

```rust
// src/types/projection.rs

use crate::types::{PlayerId, Gameweek, TeamCode};

/// Expected points and minutes for a player in a specific gameweek
#[derive(Debug, Clone)]
pub struct PlayerGameweekProjection {
    pub player_id: PlayerId,
    pub gameweek: Gameweek,
    pub expected_points: f64,      // points_md
    pub expected_minutes: f64,     // xmins_md
    pub opponent_team: Option<TeamCode>,
    pub is_home: Option<bool>,
}

/// Collection of all projections for optimization
#[derive(Debug, Clone)]
pub struct ProjectionData {
    /// Indexed by (player_id, gameweek) for O(1) lookup
    projections: HashMap<(PlayerId, Gameweek), PlayerGameweekProjection>,
    
    /// All player IDs in the dataset
    player_ids: Vec<PlayerId>,
    
    /// Gameweeks covered
    gameweeks: Vec<Gameweek>,
}

impl ProjectionData {
    pub fn get(&self, player_id: PlayerId, gw: Gameweek) -> Option<&PlayerGameweekProjection> {
        self.projections.get(&(player_id, gw))
    }
    
    pub fn players(&self) -> &[PlayerId] {
        &self.player_ids
    }
    
    pub fn gameweeks(&self) -> &[Gameweek] {
        &self.gameweeks
    }
}
```

---

## Solver Backend Selection

### **<UNCLEAR, CLARIFY>**: Which primary solver backend should we target?

**Options:**

| Backend | Crate | License | Performance | Maturity |
|---------|-------|---------|-------------|----------|
| **CBC** | `coin_cbc` | EPL 2.0 | ⭐⭐⭐ | High |
| **HiGHS** | `highs` | MIT | ⭐⭐⭐⭐ | Medium |
| **Gurobi** | FFI wrapper | Commercial | ⭐⭐⭐⭐⭐ | High |
| **good_lp** | `good_lp` | MIT | Variable | High (abstraction) |

### Tradeoff Analysis

#### Option 1: Direct `coin_cbc` binding (Recommended for MVP)

**Pros:**
- Same solver as Python implementation → consistent results
- Well-tested crate (`coin_cbc = "0.1"`)
- Free, no licensing concerns

**Cons:**
- Requires CBC installed on system
- 5-10× slower than commercial solvers
- API somewhat low-level

#### Option 2: `good_lp` abstraction layer

**Pros:**
- Solver-agnostic (can swap CBC/HiGHS/Gurobi)
- Higher-level API
- Easy to switch backends

**Cons:**
- Additional abstraction layer overhead
- May not expose all solver features
- Less control over MPS generation

#### Option 3: HiGHS via `highs` crate

**Pros:**
- Modern solver, MIT licensed
- Often faster than CBC
- Active development

**Cons:**
- Different solver → may have different optimal solutions
- Less mature Rust bindings

### Recommended Approach

**Phase 1:** Use `good_lp` with CBC backend for MVP
**Phase 2:** Add direct HiGHS support for performance
**Phase 3:** Optional Gurobi wrapper for commercial users

### Solver Trait Definition

```rust
// src/solver/traits.rs

use crate::model::Model;
use crate::solver::Solution;
use crate::error::SolverError;

/// Abstract solver interface
pub trait Solver {
    /// Solve the model and return a solution
    fn solve(&self, model: &Model) -> Result<Solution, SolverError>;
    
    /// Export model to MPS format
    fn export_mps(&self, model: &Model, path: &Path) -> Result<(), SolverError>;
    
    /// Set time limit in seconds
    fn set_time_limit(&mut self, seconds: u64);
    
    /// Get solver name for logging
    fn name(&self) -> &str;
}

/// CBC solver implementation
pub struct CbcSolver {
    time_limit: Option<u64>,
    // Additional CBC-specific options
}

impl Solver for CbcSolver {
    fn solve(&self, model: &Model) -> Result<Solution, SolverError> {
        // Implementation using coin_cbc
        todo!()
    }
    
    fn export_mps(&self, model: &Model, path: &Path) -> Result<(), SolverError> {
        // MPS export implementation
        todo!()
    }
    
    fn set_time_limit(&mut self, seconds: u64) {
        self.time_limit = Some(seconds);
    }
    
    fn name(&self) -> &str {
        "CBC"
    }
}
```

---

## Constraint Modeling

### Constraint Trait

```rust
// src/model/constraint.rs

use crate::types::PlayerId;
use crate::data::ProjectionData;

/// A constraint that can be added to the optimization model
pub trait Constraint {
    /// Unique identifier for this constraint type
    fn name(&self) -> &'static str;
    
    /// Add this constraint to the model
    fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) -> Result<(), ConstraintError>;
    
    /// Whether this constraint is required (hard) or optional (soft)
    fn is_required(&self) -> bool {
        true
    }
}

/// Data required for constraint construction
pub struct ProblemData<'a> {
    pub players: &'a [Player],
    pub projections: &'a ProjectionData,
    pub config: &'a Config,
    pub team_codes: &'a [TeamCode],
    pub positions: &'a [Position],
    pub gameweeks: &'a [Gameweek],
}
```

### Constraint Implementations

#### Squad Size Constraint

```rust
// src/constraints/squad_size.rs

pub struct SquadSizeConstraint {
    pub size: u8,  // Default: 15
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
    
    fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) -> Result<(), ConstraintError> {
        // Σ z[i] = 15
        let squad_vars = builder.get_squad_variables();
        builder.add_constraint(
            squad_vars.iter().sum::<Expression>().eq(self.size as f64),
            self.name(),
        )
    }
}
```

#### Budget Constraint

```rust
// src/constraints/budget.rs

pub struct BudgetConstraint {
    pub budget: Cost,
}

impl Default for BudgetConstraint {
    fn default() -> Self {
        Self { budget: Cost::BUDGET_DEFAULT }
    }
}

impl Constraint for BudgetConstraint {
    fn name(&self) -> &'static str {
        "total_cost"
    }
    
    fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) -> Result<(), ConstraintError> {
        // Σ z[i] × cost[i] ≤ budget
        let expr: Expression = data.players.iter()
            .map(|p| builder.squad_var(p.id) * p.cost.0 as f64)
            .sum();
        
        builder.add_constraint(
            expr.leq(self.budget.0 as f64),
            self.name(),
        )
    }
}
```

#### Team Limit Constraint

```rust
// src/constraints/team_limit.rs

pub struct TeamLimitConstraint {
    pub max_per_team: u8,  // Default: 3
}

impl Default for TeamLimitConstraint {
    fn default() -> Self {
        Self { max_per_team: 3 }
    }
}

impl Constraint for TeamLimitConstraint {
    fn name(&self) -> &'static str {
        "player_team_limit"
    }
    
    fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) -> Result<(), ConstraintError> {
        // For each team j: Σ z[i] for i where team[i]=j ≤ 3
        for team_code in data.team_codes {
            let team_players: Expression = data.players.iter()
                .filter(|p| p.team_code == *team_code)
                .map(|p| builder.squad_var(p.id))
                .sum();
            
            builder.add_constraint(
                team_players.leq(self.max_per_team as f64),
                &format!("team_limit_{}", team_code.0),
            )?;
        }
        Ok(())
    }
}
```

#### Position Constraints

```rust
// src/constraints/position.rs

/// Exact squad composition constraint (2 GK, 5 DEF, 5 MID, 3 FWD)
pub struct SquadPositionConstraint;

impl Constraint for SquadPositionConstraint {
    fn name(&self) -> &'static str {
        "squad_exact"
    }
    
    fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) -> Result<(), ConstraintError> {
        for pos in Position::all() {
            let pos_players: Expression = data.players.iter()
                .filter(|p| p.position == pos)
                .map(|p| builder.squad_var(p.id))
                .sum();
            
            builder.add_constraint(
                pos_players.eq(pos.squad_select() as f64),
                &format!("squad_{:?}", pos),
            )?;
        }
        Ok(())
    }
}

/// Lineup position bounds (min/max starters per position)
pub struct LineupPositionConstraint;

impl Constraint for LineupPositionConstraint {
    fn name(&self) -> &'static str {
        "lineup_position"
    }
    
    fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) -> Result<(), ConstraintError> {
        for pos in Position::all() {
            let pos_lineup: Expression = data.players.iter()
                .filter(|p| p.position == pos)
                .map(|p| builder.lineup_var(p.id))
                .sum();
            
            // Min constraint
            builder.add_constraint(
                pos_lineup.clone().geq(pos.min_play() as f64),
                &format!("lineup_min_{:?}", pos),
            )?;
            
            // Max constraint
            builder.add_constraint(
                pos_lineup.leq(pos.max_play() as f64),
                &format!("lineup_max_{:?}", pos),
            )?;
        }
        Ok(())
    }
}
```

#### Differential Constraint

```rust
// src/constraints/differential.rs

pub struct DifferentialConstraint {
    pub max_ownership_percent: f64,  // Default: 5.0
}

impl Default for DifferentialConstraint {
    fn default() -> Self {
        Self { max_ownership_percent: 5.0 }
    }
}

impl Constraint for DifferentialConstraint {
    fn name(&self) -> &'static str {
        "allow_only_differentials"
    }
    
    fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) -> Result<(), ConstraintError> {
        // z[i] = 0 for all players where ownership > threshold
        for player in data.players {
            if player.ownership_percent > self.max_ownership_percent {
                builder.add_constraint(
                    builder.squad_var(player.id).eq(0.0),
                    &format!("diff_ban_{}", player.id.0),
                )?;
            }
        }
        Ok(())
    }
}
```

---

## Problem Implementations

### Problem Trait

```rust
// src/problems/traits.rs

use crate::model::{ModelBuilder, Objective};
use crate::constraints::Constraint;
use crate::solver::Solution;

/// Defines a complete optimization problem
pub trait Problem {
    /// Human-readable problem name
    fn name(&self) -> &str;
    
    /// File name for output (without extension)
    fn output_name(&self) -> &str;
    
    /// Required constraints for this problem
    fn constraints(&self) -> Vec<Box<dyn Constraint>>;
    
    /// Objective function configuration
    fn objective(&self) -> Objective;
    
    /// Parse solution into human-readable format
    fn format_solution(&self, solution: &Solution, data: &ProblemData) -> ProblemResult;
}

/// Standard objective function types
pub enum Objective {
    /// Maximize expected points: Σ points[i] × (x[i] + y[i])
    MaximizeExpectedPoints,
    
    /// With bench weight: Σ points[i] × (x[i] + y[i] + w×(z[i] - x[i]))
    MaximizeWithBenchWeight { bench_weight: f64 },
    
    /// Bench boost: Σ points[i] × (z[i] + y[i])
    MaximizeBenchBoost,
    
    /// Custom objective expression
    Custom(Box<dyn Fn(&mut ModelBuilder, &ProblemData) -> Expression>),
}
```

### Limited Best Squad Implementation

```rust
// src/problems/limited_best_squad.rs

use crate::problems::traits::{Problem, Objective};
use crate::constraints::*;

/// Standard FPL team selection with all constraints
pub struct LimitedBestSquad {
    pub budget: Cost,
    pub team_limit: u8,
}

impl Default for LimitedBestSquad {
    fn default() -> Self {
        Self {
            budget: Cost::BUDGET_DEFAULT,
            team_limit: 3,
        }
    }
}

impl Problem for LimitedBestSquad {
    fn name(&self) -> &str {
        "Limited Best Squad"
    }
    
    fn output_name(&self) -> &str {
        "limited_best_15"
    }
    
    fn constraints(&self) -> Vec<Box<dyn Constraint>> {
        vec![
            Box::new(SquadSizeConstraint::default()),
            Box::new(LineupSizeConstraint::default()),
            Box::new(BudgetConstraint { budget: self.budget }),
            Box::new(TeamLimitConstraint { max_per_team: self.team_limit }),
            Box::new(SquadPositionConstraint),
            Box::new(LineupPositionConstraint),
            Box::new(CaptainConstraint),
            Box::new(LineupSquadConstraint),
        ]
    }
    
    fn objective(&self) -> Objective {
        Objective::MaximizeExpectedPoints
    }
    
    fn format_solution(&self, solution: &Solution, data: &ProblemData) -> ProblemResult {
        // Parse binary variables to select players
        let mut squad = Vec::new();
        let mut lineup = Vec::new();
        let mut captain_id = None;
        
        for player in data.players {
            let in_squad = solution.get_value(&format!("squad_{}", player.id.0)) > 0.5;
            let in_lineup = solution.get_value(&format!("lineup_{}", player.id.0)) > 0.5;
            let is_captain = solution.get_value(&format!("captain_{}", player.id.0)) > 0.5;
            
            if in_squad {
                squad.push(player.id);
            }
            if in_lineup {
                lineup.push(player.id);
            }
            if is_captain {
                captain_id = Some(player.id);
            }
        }
        
        ProblemResult {
            squad,
            lineup,
            captain: captain_id,
            bench: squad.iter()
                .filter(|id| !lineup.contains(id))
                .cloned()
                .collect(),
            objective_value: solution.objective_value(),
        }
    }
}
```

### Iterative Squads Implementation

```rust
// src/problems/iterative.rs

use crate::problems::traits::Problem;
use crate::solver::Solver;

/// Configuration for iterative squad generation
pub struct IterativeSquadsConfig {
    pub total_iterations: usize,
    pub squad_cutoff: usize,  // Max overlap with previous solution
    pub random_seed: Option<u64>,
}

impl Default for IterativeSquadsConfig {
    fn default() -> Self {
        Self {
            total_iterations: 50,
            squad_cutoff: 12,
            random_seed: Some(42),
        }
    }
}

/// Generates multiple diverse squad solutions
pub struct IterativeSquads {
    pub config: IterativeSquadsConfig,
    pub base_problem: LimitedBestSquad,
}

impl IterativeSquads {
    /// Run iterative solving
    pub fn solve<S: Solver>(
        &self,
        solver: &S,
        data: &ProblemData,
    ) -> Result<Vec<ProblemResult>, SolverError> {
        let mut results = Vec::with_capacity(self.config.total_iterations);
        let mut rng = match self.config.random_seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };
        
        for iteration in 0..self.config.total_iterations {
            let mut builder = ModelBuilder::new();
            
            // Apply base constraints
            for constraint in self.base_problem.constraints() {
                constraint.apply(&mut builder, data)?;
            }
            
            // Add cutoff constraint from previous solutions
            if iteration > 0 {
                let prev_squad = &results[iteration - 1].squad;
                // Σ z[i] for i in prev_squad ≤ cutoff
                let cutoff_expr: Expression = prev_squad.iter()
                    .map(|id| builder.squad_var(*id))
                    .sum();
                builder.add_constraint(
                    cutoff_expr.leq(self.config.squad_cutoff as f64),
                    &format!("cutoff_{}", iteration),
                )?;
            }
            
            // Randomized objective weights (for diversity)
            self.apply_randomized_objective(&mut builder, data, &mut rng)?;
            
            // Solve
            let solution = solver.solve(&builder.build())?;
            results.push(self.base_problem.format_solution(&solution, data));
        }
        
        Ok(results)
    }
    
    fn apply_randomized_objective(
        &self,
        builder: &mut ModelBuilder,
        data: &ProblemData,
        rng: &mut StdRng,
    ) -> Result<(), ConstraintError> {
        // Weighted combination of multiple objectives
        let w_total: f64 = rng.gen();
        let w_cost: f64 = rng.gen();
        let w_xmin: f64 = rng.gen();
        let w_form: f64 = rng.gen();
        let w_ownership: f64 = rng.gen_range(-1.0..1.0);
        
        // Build composite objective
        // **<UNCLEAR, CLARIFY>**: Should we normalize weights? Current Python doesn't.
        todo!("Implement weighted objective composition")
    }
}
```

---

## API Design

### Builder Pattern for Model Construction

```rust
// src/model/builder.rs

pub struct ModelBuilder {
    /// Lineup variables: x[player_id]
    lineup_vars: HashMap<PlayerId, Variable>,
    /// Captain variables: y[player_id]
    captain_vars: HashMap<PlayerId, Variable>,
    /// Squad variables: z[player_id]
    squad_vars: HashMap<PlayerId, Variable>,
    /// Constraints
    constraints: Vec<(Expression, Relation, f64, String)>,
    /// Objective
    objective: Option<Expression>,
    /// Minimize (false) or Maximize (true)
    maximize: bool,
}

impl ModelBuilder {
    pub fn new() -> Self {
        Self {
            lineup_vars: HashMap::new(),
            captain_vars: HashMap::new(),
            squad_vars: HashMap::new(),
            constraints: Vec::new(),
            objective: None,
            maximize: true,
        }
    }
    
    /// Initialize variables for all players
    pub fn init_variables(&mut self, players: &[Player]) {
        for player in players {
            self.lineup_vars.insert(
                player.id,
                Variable::new_binary(format!("lineup_{}", player.id.0)),
            );
            self.captain_vars.insert(
                player.id,
                Variable::new_binary(format!("captain_{}", player.id.0)),
            );
            self.squad_vars.insert(
                player.id,
                Variable::new_binary(format!("squad_{}", player.id.0)),
            );
        }
    }
    
    pub fn lineup_var(&self, id: PlayerId) -> &Variable {
        self.lineup_vars.get(&id).expect("Player not initialized")
    }
    
    pub fn captain_var(&self, id: PlayerId) -> &Variable {
        self.captain_vars.get(&id).expect("Player not initialized")
    }
    
    pub fn squad_var(&self, id: PlayerId) -> &Variable {
        self.squad_vars.get(&id).expect("Player not initialized")
    }
    
    pub fn add_constraint(
        &mut self,
        constraint: impl Into<ConstraintExpr>,
        name: &str,
    ) -> Result<(), ConstraintError> {
        // Validate and add constraint
        let expr = constraint.into();
        self.constraints.push((expr.lhs, expr.relation, expr.rhs, name.to_string()));
        Ok(())
    }
    
    pub fn set_objective(&mut self, expr: Expression, maximize: bool) {
        self.objective = Some(expr);
        self.maximize = maximize;
    }
    
    pub fn build(self) -> Model {
        Model {
            variables: self.collect_variables(),
            constraints: self.constraints,
            objective: self.objective.expect("Objective not set"),
            maximize: self.maximize,
        }
    }
    
    fn collect_variables(&self) -> Vec<Variable> {
        let mut vars = Vec::new();
        vars.extend(self.lineup_vars.values().cloned());
        vars.extend(self.captain_vars.values().cloned());
        vars.extend(self.squad_vars.values().cloned());
        vars
    }
}
```

### CLI Interface

```rust
// src/main.rs

use clap::{Parser, Subcommand};
use fpl_optimizer::{
    problems::*,
    solver::CbcSolver,
    data::FplDataLoader,
    config::Config,
};

#[derive(Parser)]
#[command(name = "fpl-optimizer")]
#[command(about = "FPL Squad Optimization using MILP")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,
    
    /// Output directory
    #[arg(short, long, default_value = "output")]
    output: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Solve standard limited best squad
    Solve {
        /// Gameweek to optimize for
        #[arg(short, long)]
        gameweek: Option<u8>,
        
        /// Problem type
        #[arg(short, long, default_value = "limited")]
        problem: ProblemType,
        
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
    
    /// Export model to MPS format without solving
    Export {
        /// Output MPS file path
        #[arg(short, long)]
        output: PathBuf,
        
        /// Problem type
        #[arg(short, long, default_value = "limited")]
        problem: ProblemType,
    },
    
    /// Fetch latest data from FPL API
    Update,
}

#[derive(Clone, ValueEnum)]
enum ProblemType {
    NoLimit,
    Limited,
    WeightedBench,
    BenchBoost,
    Differential,
    SetAndForget,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = Config::load(&cli.config)?;
    
    match cli.command {
        Commands::Solve { gameweek, problem, budget } => {
            let data = FplDataLoader::load(&config)?;
            let solver = CbcSolver::new();
            
            let problem_instance: Box<dyn Problem> = match problem {
                ProblemType::NoLimit => Box::new(NoLimitBest11),
                ProblemType::Limited => Box::new(LimitedBestSquad {
                    budget: budget.map(Cost::from_millions).unwrap_or(Cost::BUDGET_DEFAULT),
                    ..Default::default()
                }),
                ProblemType::WeightedBench => Box::new(WeightedBenchSquad::default()),
                ProblemType::BenchBoost => Box::new(BenchBoostSquad::default()),
                ProblemType::Differential => Box::new(DifferentialSquad::default()),
                ProblemType::SetAndForget => Box::new(SetAndForget::default()),
            };
            
            let result = solve_problem(&*problem_instance, &solver, &data)?;
            result.write_csv(&cli.output.join(format!("{}.csv", problem_instance.output_name())))?;
            
            println!("{}", result.display());
        }
        
        Commands::Iterative { count, gameweek } => {
            todo!("Iterative solving")
        }
        
        Commands::Export { output, problem } => {
            todo!("MPS export")
        }
        
        Commands::Update => {
            todo!("Data update")
        }
    }
    
    Ok(())
}
```

---

## Configuration System

### TOML Configuration

```toml
# config.toml

[general]
season = "2025-26"
data_dir = "./data"

[solver]
backend = "cbc"  # or "highs", "gurobi"
time_limit_seconds = 60
threads = 4
mip_gap = 0.0001

[constraints]
budget = 1000           # In tenths of millions
team_limit = 3
lineup_size = 11
squad_size = 15

[bench]
weight = 0.1            # Bench weight for weighted problems

[differential]
max_ownership = 5.0     # Maximum ownership percentage

[iterative]
iterations = 50
cutoff = 12
random_seed = 42

[output]
format = "csv"          # or "json"
include_mps = true
```

### Config Struct

```rust
// src/config/params.rs

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub solver: SolverConfig,
    pub constraints: ConstraintConfig,
    pub bench: BenchConfig,
    pub differential: DifferentialConfig,
    pub iterative: IterativeConfig,
    pub output: OutputConfig,
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub season: String,
    pub data_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct SolverConfig {
    pub backend: SolverBackend,
    pub time_limit_seconds: u64,
    pub threads: u8,
    pub mip_gap: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SolverBackend {
    Cbc,
    Highs,
    Gurobi,
}

#[derive(Debug, Deserialize)]
pub struct ConstraintConfig {
    pub budget: u16,
    pub team_limit: u8,
    pub lineup_size: u8,
    pub squad_size: u8,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
```

---

## Testing Strategy

### Unit Tests

```rust
// tests/constraint_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    
    #[test]
    fn test_budget_constraint_enforced() {
        let players = create_test_players(20);
        let mut builder = ModelBuilder::new();
        builder.init_variables(&players);
        
        let budget = BudgetConstraint { budget: Cost(500) };
        budget.apply(&mut builder, &create_test_data(&players)).unwrap();
        
        // Verify constraint was added
        assert!(builder.has_constraint("total_cost"));
    }
    
    #[test]
    fn test_squad_size_exactly_15() {
        let players = create_test_players(20);
        let mut builder = ModelBuilder::new();
        builder.init_variables(&players);
        
        let constraint = SquadSizeConstraint::default();
        constraint.apply(&mut builder, &create_test_data(&players)).unwrap();
        
        // Solve and verify
        let solver = CbcSolver::new();
        let model = builder
            .set_objective(trivial_objective(&players), true)
            .build();
        let solution = solver.solve(&model).unwrap();
        
        let squad_count: usize = players.iter()
            .filter(|p| solution.get_value(&format!("squad_{}", p.id.0)) > 0.5)
            .count();
        
        assert_eq!(squad_count, 15);
    }
    
    #[test]
    fn test_position_limits_respected() {
        // Test that GK=2, DEF=5, MID=5, FWD=3 in squad
        todo!()
    }
    
    #[test]
    fn test_captain_must_be_in_lineup() {
        // Test y[i] <= x[i] constraint
        todo!()
    }
}
```

### Integration Tests

```rust
// tests/problem_tests.rs

#[test]
fn test_limited_best_squad_produces_valid_solution() {
    let config = Config::default();
    let data = load_test_data();
    let solver = CbcSolver::new();
    
    let problem = LimitedBestSquad::default();
    let result = solve_problem(&problem, &solver, &data).unwrap();
    
    // Validate solution
    assert_eq!(result.squad.len(), 15);
    assert_eq!(result.lineup.len(), 11);
    assert!(result.captain.is_some());
    assert!(result.lineup.contains(&result.captain.unwrap()));
    
    // Validate budget
    let total_cost: Cost = result.squad.iter()
        .map(|id| data.players.get(id).unwrap().cost)
        .sum();
    assert!(total_cost <= Cost::BUDGET_DEFAULT);
    
    // Validate team limits
    let team_counts = count_by_team(&result.squad, &data);
    assert!(team_counts.values().all(|&c| c <= 3));
}

#[test]
fn test_differential_excludes_high_ownership() {
    let config = Config::default();
    let data = load_test_data();
    let solver = CbcSolver::new();
    
    let problem = DifferentialSquad { max_ownership: 5.0 };
    let result = solve_problem(&problem, &solver, &data).unwrap();
    
    for player_id in &result.squad {
        let player = data.players.get(player_id).unwrap();
        assert!(
            player.ownership_percent <= 5.0,
            "Player {} has {}% ownership (> 5%)",
            player.web_name,
            player.ownership_percent
        );
    }
}
```

### Benchmark Tests

```rust
// benches/solver_benchmark.rs

use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_limited_squad(c: &mut Criterion) {
    let data = load_benchmark_data();
    let solver = CbcSolver::new();
    let problem = LimitedBestSquad::default();
    
    c.bench_function("limited_squad_solve", |b| {
        b.iter(|| {
            solve_problem(&problem, &solver, &data).unwrap()
        })
    });
}

fn benchmark_iterative_50(c: &mut Criterion) {
    let data = load_benchmark_data();
    let solver = CbcSolver::new();
    let config = IterativeSquadsConfig {
        total_iterations: 50,
        ..Default::default()
    };
    let problem = IterativeSquads { config, base_problem: LimitedBestSquad::default() };
    
    c.bench_function("iterative_50_squads", |b| {
        b.iter(|| {
            problem.solve(&solver, &data).unwrap()
        })
    });
}

criterion_group!(benches, benchmark_limited_squad, benchmark_iterative_50);
criterion_main!(benches);
```

---

## Performance Considerations

### Memory Optimization

1. **Variable Pooling**: Reuse variable IDs across iterations to reduce allocations
2. **Sparse Constraint Representation**: Use sparse matrices for constraint coefficients
3. **Arena Allocation**: Consider using `bumpalo` for short-lived allocations during model building

### Parallelization Strategy

```rust
// Parallel iterative solving using rayon

use rayon::prelude::*;

impl IterativeSquads {
    pub fn solve_parallel<S: Solver + Sync>(
        &self,
        solver: &S,
        data: &ProblemData,
    ) -> Result<Vec<ProblemResult>, SolverError> {
        // **<UNCLEAR, CLARIFY>**: Can we parallelize iterative solving?
        // Current Python implementation is sequential due to cutoff constraints
        // depending on previous solutions.
        //
        // Option 1: Keep sequential (same as Python)
        // Option 2: Generate initial diverse seeds, then parallel refinement
        // Option 3: Independent parallel runs with different random seeds
        
        // For now, implement sequential to match Python behavior
        self.solve_sequential(solver, data)
    }
}
```

### Solver Warm Starting

```rust
impl CbcSolver {
    /// Provide an initial solution to warm-start the solver
    pub fn set_initial_solution(&mut self, solution: &Solution) {
        // **<UNCLEAR, CLARIFY>**: Does coin_cbc support warm starting?
        // Need to verify API capabilities
        todo!()
    }
}
```

---

## Implementation Roadmap

### Phase 1: Core Infrastructure (Week 1-2)

- [ ] **TODO 1.1**: Set up Cargo project with dependencies
- [ ] **TODO 1.2**: Implement core types (`Player`, `Position`, `Cost`, etc.)
- [ ] **TODO 1.3**: Implement data loading from CSV
- [ ] **TODO 1.4**: Basic model builder with variable creation
- [ ] **TODO 1.5**: CBC solver integration via `coin_cbc` or `good_lp`
- [ ] **TODO 1.6**: MPS export functionality

### Phase 2: Basic Problems (Week 2-3)

- [ ] **TODO 2.1**: Implement all base constraints
- [ ] **TODO 2.2**: `NoLimitBest11` problem
- [ ] **TODO 2.3**: `LimitedBestSquad` problem
- [ ] **TODO 2.4**: CSV output formatting
- [ ] **TODO 2.5**: Unit tests for constraints
- [ ] **TODO 2.6**: Integration tests for problems

### Phase 3: Advanced Problems (Week 3-4)

- [ ] **TODO 3.1**: `WeightedBenchSquad` problem
- [ ] **TODO 3.2**: `BenchBoostSquad` problem
- [ ] **TODO 3.3**: `DifferentialSquad` problem
- [ ] **TODO 3.4**: `SetAndForget` multi-gameweek problem
- [ ] **TODO 3.5**: Iterative squad generation

### Phase 4: Data Integration (Week 4-5)

- [ ] **TODO 4.1**: FPL API client (async with `reqwest`)
- [ ] **TODO 4.2**: Projection data parsing
- [ ] **TODO 4.3**: Intermediate layer generation
- [ ] **TODO 4.4**: Data caching

### Phase 5: Polish & Optimization (Week 5-6)

- [ ] **TODO 5.1**: CLI interface with `clap`
- [ ] **TODO 5.2**: Configuration system
- [ ] **TODO 5.3**: HiGHS solver backend
- [ ] **TODO 5.4**: Performance benchmarks
- [ ] **TODO 5.5**: Documentation
- [ ] **TODO 5.6**: Error handling improvements

---

## Appendix: Mathematical Formulation

### Decision Variables

| Variable | Type | Description |
|----------|------|-------------|
| `x[i]` | Binary | Player `i` is in starting lineup |
| `y[i]` | Binary | Player `i` is captain |
| `z[i]` | Binary | Player `i` is in squad |
| `bench[i,p]` | Binary | Player `i` is in bench position `p` (iterative only) |

### Objective Functions

**Standard:**
```
maximize: Σᵢ (points[i] × (x[i] + y[i]))
```

**Weighted Bench:**
```
maximize: Σᵢ (points[i] × (x[i] + y[i] + 0.1×(z[i] - x[i])))
```

**Bench Boost:**
```
maximize: Σᵢ (points[i] × (z[i] + y[i]))
```

### Constraints

| Name | Formula | Description |
|------|---------|-------------|
| Lineup Size | `Σᵢ x[i] = 11` | Exactly 11 starters |
| Squad Size | `Σᵢ z[i] = 15` | Exactly 15 in squad |
| Budget | `Σᵢ z[i]×cost[i] ≤ B` | Total cost within budget |
| Team Limit | `Σᵢ∈T z[i] ≤ 3` ∀T | Max 3 from each team |
| Captain | `Σᵢ y[i] = 1` | Exactly 1 captain |
| Captain Plays | `y[i] ≤ x[i]` ∀i | Captain must be in lineup |
| Lineup ⊆ Squad | `x[i] ≤ z[i]` ∀i | Lineup players must be in squad |
| Position Min | `Σᵢ∈P x[i] ≥ min(P)` ∀P | Minimum per position |
| Position Max | `Σᵢ∈P x[i] ≤ max(P)` ∀P | Maximum per position |
| Squad Exact | `Σᵢ∈P z[i] = select(P)` ∀P | Exact squad composition |

---

## Appendix: Dependency Recommendations

```toml
# Cargo.toml

[package]
name = "fpl-optimizer"
version = "0.1.0"
edition = "2021"

[dependencies]
# Solver backends
good_lp = { version = "1.8", features = ["coin_cbc"] }
# coin_cbc = "0.1"  # Alternative: direct CBC binding
# highs = "1.5"     # Alternative: HiGHS solver

# Async runtime for API calls
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }

# Data handling
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
csv = "1.3"
toml = "0.8"

# CLI
clap = { version = "4", features = ["derive"] }

# Utilities
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
rand = "0.8"

[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "solver_benchmark"
harness = false
```

---

## Open Questions & Clarifications Needed

### **<UNCLEAR, CLARIFY>**: Multi-period variable indexing

The Python implementation uses `(player_id, gameweek)` tuples for multi-period problems. In Rust, should we:
1. Use nested HashMaps `HashMap<PlayerId, HashMap<Gameweek, Variable>>`
2. Use a single flat HashMap with tuple keys `HashMap<(PlayerId, Gameweek), Variable>`
3. Use a custom struct with Index trait implementation

**Recommendation:** Option 2 for simplicity and O(1) lookup.

---

### **<UNCLEAR, CLARIFY>**: Error handling granularity

What level of error detail is needed?
1. Simple error types with strings
2. Detailed error enums for every failure mode
3. Error chain with context (using `anyhow`)

**Recommendation:** Use `thiserror` for library errors, `anyhow` at the application boundary.

---

### **<UNCLEAR, CLARIFY>**: FPL API rate limiting

The Python implementation uses `time.sleep()` between API calls. Should we:
1. Implement the same naive sleeping
2. Use a proper rate limiter (`governor` crate)
3. Use exponential backoff with jitter

**Recommendation:** Use `governor` for production-quality rate limiting.

---

### **<UNCLEAR, CLARIFY>**: Solution validation

Should solution validation be:
1. Opt-in (user calls `validate()`)
2. Always-on in debug builds
3. Always-on in all builds

**Recommendation:** Always-on in debug, opt-in in release via feature flag.

---

### **<UNCLEAR, CLARIFY>**: MPS format compatibility

The Python code massages the MPS output for compatibility. Need to verify:
1. Does `good_lp` produce compatible MPS?
2. Do we need the same field adjustments?
3. Should we add CBC-specific MPS options?

**Action:** Test with CBC and document any adjustments needed.

---

## Document Version

**Version:** 1.0.0  
**Last Updated:** 2026-01-08  
**Author:** Implementation Guide for Rust Port

---

## References

1. Original Python implementation: `/workspace/src/solve.py`
2. Specification: `/workspace/SPECIFICATION.md`
3. Parameter Guide: `/workspace/PARAMETER_GUIDE.md`
4. `good_lp` documentation: https://docs.rs/good_lp
5. `coin_cbc` documentation: https://docs.rs/coin_cbc
6. FPL API: https://fantasy.premierleague.com/api/bootstrap-static/
