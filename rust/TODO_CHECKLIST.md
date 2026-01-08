# FPL Optimizer Rust Implementation - TODO Checklist

## Overview

This checklist tracks all implementation tasks. Check off items as they're completed.

**Legend:**
- 🔴 Not Started
- 🟡 In Progress  
- 🟢 Complete
- ⚠️ Blocked
- **<UNCLEAR, CLARIFY>** Needs clarification

---

## Phase 1: Project Setup & Core Types (Week 1)

### 1.1 Project Infrastructure
- [ ] 🔴 Initialize Cargo workspace
- [ ] 🔴 Set up GitHub Actions CI/CD
- [ ] 🔴 Configure rustfmt and clippy
- [ ] 🔴 Set up pre-commit hooks
- [ ] 🔴 Create README.md with quickstart

### 1.2 Core Types Implementation

#### Player Types (`src/types/player.rs`)
- [ ] 🔴 `PlayerId` newtype with `Display`, `FromStr`
- [ ] 🔴 `Player` struct with all fields
- [ ] 🔴 Implement `Hash`, `Eq` for `PlayerId`
- [ ] 🔴 Unit tests for player type

#### Team Types (`src/types/team.rs`)
- [ ] 🔴 `TeamCode` newtype
- [ ] 🔴 `Team` struct with code and name
- [ ] 🔴 Team name lookup table (20 PL teams)

#### Position Types (`src/types/position.rs`)
- [ ] 🔴 `Position` enum (GK, DEF, MID, FWD)
- [ ] 🔴 `squad_select()`, `min_play()`, `max_play()` methods
- [ ] 🔴 `FromStr` and `Display` implementations
- [ ] 🔴 `Position::all()` iterator
- [ ] 🔴 Unit tests for position constraints

#### Cost Types (`src/types/cost.rs`)
- [ ] 🔴 `Cost` newtype (u16 in tenths of millions)
- [ ] 🔴 `BUDGET_DEFAULT` constant
- [ ] 🔴 `from_millions()` and `to_millions()` converters
- [ ] 🔴 Arithmetic trait implementations (Add, Sub, Sum)
- [ ] 🔴 Unit tests for cost arithmetic

#### Gameweek Types (`src/types/gameweek.rs`)
- [ ] 🔴 `Gameweek` newtype (u8, 1-38)
- [ ] 🔴 Validation (must be 1-38)
- [ ] 🔴 Range iteration support

#### Projection Types (`src/types/projection.rs`)
- [ ] 🔴 `PlayerGameweekProjection` struct
- [ ] 🔴 `ProjectionData` collection with efficient lookup
- [ ] 🔴 Builder pattern for `ProjectionData`

### 1.3 Error Types (`src/error.rs`)
- [ ] 🔴 `FplError` root error enum
- [ ] 🔴 `DataError` for data loading failures
- [ ] 🔴 `SolverError` for solver failures
- [ ] 🔴 `ConstraintError` for constraint building failures
- [ ] 🔴 `ConfigError` for configuration issues
- [ ] 🔴 Implement `std::error::Error` for all types

---

## Phase 2: Data Loading (Week 1-2)

### 2.1 CSV Loader (`src/data/csv_loader.rs`)
- [ ] 🔴 Load `element.csv` (player data)
- [ ] 🔴 Load `element_type.csv` (position rules)
- [ ] 🔴 Load `team.csv` (team data)
- [ ] 🔴 Load `element_gameweek.csv` (projections)
- [ ] 🔴 Load custom projection CSV (`gw{N}.csv`)
- [ ] 🔴 Error handling for missing/malformed files
- [ ] 🔴 Integration tests with sample data

### 2.2 FPL API Client (`src/data/fpl_api.rs`)
- [ ] 🔴 Async HTTP client setup with `reqwest`
- [ ] 🔴 Bootstrap static endpoint (`/api/bootstrap-static/`)
- [ ] 🔴 Fixtures endpoint (`/api/fixtures/`)
- [ ] 🔴 Live gameweek endpoint (`/api/event/{GW}/live/`)
- [ ] 🔴 Rate limiting with `governor`
- [ ] 🔴 Retry logic with exponential backoff
- [ ] 🔴 Response caching
- [ ] 🔴 Unit tests with `wiremock`

### 2.3 Data Schemas (`src/data/schema.rs`)
- [ ] 🔴 `ElementResponse` (player from API)
- [ ] 🔴 `TeamResponse` (team from API)
- [ ] 🔴 `ElementTypeResponse` (position from API)
- [ ] 🔴 `FixtureResponse` (fixture from API)
- [ ] 🔴 `EventResponse` (gameweek from API)
- [ ] 🔴 Serde derive implementations

### 2.4 Intermediate Layer (`src/data/intermediate.rs`)
- [ ] 🔴 Match fixtures to players
- [ ] 🔴 Merge projection data with player data
- [ ] 🔴 Generate `element_gameweek` equivalent
- [ ] 🔴 Handle missing projections gracefully
- [ ] 🔴 **<UNCLEAR, CLARIFY>**: How to handle double gameweeks?

---

## Phase 3: Model Builder (Week 2)

### 3.1 Variable Definitions (`src/model/variables.rs`)
- [ ] 🔴 `Variable` struct wrapping solver variable
- [ ] 🔴 `VariableType` enum (Binary, Integer, Continuous)
- [ ] 🔴 Variable naming convention (`lineup_{id}`, `captain_{id}`, `squad_{id}`)
- [ ] 🔴 Variable bounds support

### 3.2 Expression Building
- [ ] 🔴 `Expression` type for linear expressions
- [ ] 🔴 Operator overloading (`+`, `-`, `*` with scalars)
- [ ] 🔴 `sum()` helper for iterating over expressions
- [ ] 🔴 `Relation` enum (Eq, Leq, Geq)
- [ ] 🔴 Constraint expression helpers (`.eq()`, `.leq()`, `.geq()`)

### 3.3 Model Builder (`src/model/builder.rs`)
- [ ] 🔴 `ModelBuilder` struct
- [ ] 🔴 `init_variables()` for player variables
- [ ] 🔴 Variable accessors (`lineup_var()`, `captain_var()`, `squad_var()`)
- [ ] 🔴 `add_constraint()` method
- [ ] 🔴 `set_objective()` method
- [ ] 🔴 `build()` to finalize model
- [ ] 🔴 Multi-gameweek variable support
- [ ] 🔴 Bench position variables (for iterative)

### 3.4 Objective Functions (`src/model/objective.rs`)
- [ ] 🔴 `Objective` enum (MaximizeExpectedPoints, etc.)
- [ ] 🔴 Standard objective builder: Σ points × (x + y)
- [ ] 🔴 Weighted bench objective: Σ points × (x + y + w×(z-x))
- [ ] 🔴 Bench boost objective: Σ points × (z + y)
- [ ] 🔴 Custom objective support

---

## Phase 4: Constraint Implementations (Week 2-3)

### 4.1 Core Constraints

#### Squad Size (`src/constraints/squad_size.rs`)
- [ ] 🔴 Implement `Constraint` trait
- [ ] 🔴 `Σ z[i] = 15` constraint
- [ ] 🔴 Configurable size parameter
- [ ] 🔴 Unit test

#### Lineup Size (`src/constraints/lineup_size.rs`)
- [ ] 🔴 Implement `Constraint` trait
- [ ] 🔴 `Σ x[i] = 11` constraint
- [ ] 🔴 Unit test

#### Budget (`src/constraints/budget.rs`)
- [ ] 🔴 Implement `Constraint` trait
- [ ] 🔴 `Σ z[i] × cost[i] ≤ budget` constraint
- [ ] 🔴 Configurable budget parameter
- [ ] 🔴 Unit test

#### Team Limit (`src/constraints/team_limit.rs`)
- [ ] 🔴 Implement `Constraint` trait
- [ ] 🔴 For each team: `Σ z[i] ≤ 3`
- [ ] 🔴 Configurable limit parameter
- [ ] 🔴 Unit test

#### Position Min/Max (`src/constraints/position.rs`)
- [ ] 🔴 `SquadPositionConstraint`: exact counts in squad
- [ ] 🔴 `LineupPositionConstraint`: min/max in lineup
- [ ] 🔴 Unit tests for each position

#### Captain (`src/constraints/captain.rs`)
- [ ] 🔴 `Σ y[i] = 1` (single captain)
- [ ] 🔴 `y[i] ≤ x[i]` (captain must play)
- [ ] 🔴 Unit test

#### Lineup Squad Link (`src/constraints/lineup_squad.rs`)
- [ ] 🔴 `x[i] ≤ z[i]` for all players
- [ ] 🔴 Unit test

### 4.2 Problem-Specific Constraints

#### Differential (`src/constraints/differential.rs`)
- [ ] 🔴 `z[i] = 0` for ownership > threshold
- [ ] 🔴 Configurable ownership threshold
- [ ] 🔴 Unit test

#### Bench Ordering (`src/constraints/bench.rs`)
- [ ] 🔴 `bench[i,p]` variables
- [ ] 🔴 Each bench position filled once
- [ ] 🔴 GK must be bench position 1
- [ ] 🔴 `lineup + bench = squad`
- [ ] 🔴 **<UNCLEAR, CLARIFY>**: Is bench ordering actually used in Python?

#### Iterative Cutoff (`src/constraints/iterative.rs`)
- [ ] 🔴 Squad overlap cutoff constraint
- [ ] 🔴 Dynamic constraint addition after each iteration
- [ ] 🔴 Unit test

---

## Phase 5: Problem Implementations (Week 3)

### 5.1 Problem Trait (`src/problems/traits.rs`)
- [ ] 🔴 `Problem` trait definition
- [ ] 🔴 `ProblemResult` struct
- [ ] 🔴 Solution parsing logic

### 5.2 Individual Problems

#### No Limit Best 11 (`src/problems/no_limit_best_11.rs`)
- [ ] 🔴 Implement `Problem` trait
- [ ] 🔴 Constraints: lineup size, position min/max, captain
- [ ] 🔴 NO budget, NO team limit
- [ ] 🔴 Integration test

#### Limited Best Squad (`src/problems/limited_best_squad.rs`)
- [ ] 🔴 Implement `Problem` trait
- [ ] 🔴 All standard FPL constraints
- [ ] 🔴 Integration test
- [ ] 🔴 Verify matches Python output

#### Weighted Bench (`src/problems/weighted_bench.rs`)
- [ ] 🔴 Implement `Problem` trait
- [ ] 🔴 Modified objective with bench weight
- [ ] 🔴 Configurable bench weight (default 0.1)
- [ ] 🔴 Integration test

#### Bench Boost (`src/problems/bench_boost.rs`)
- [ ] 🔴 Implement `Problem` trait
- [ ] 🔴 Objective: Σ points × (z + y)
- [ ] 🔴 Integration test

#### Differential (`src/problems/differential.rs`)
- [ ] 🔴 Implement `Problem` trait
- [ ] 🔴 Add differential constraint
- [ ] 🔴 Configurable ownership threshold
- [ ] 🔴 Integration test

#### Set and Forget (`src/problems/set_and_forget.rs`)
- [ ] 🔴 Implement `Problem` trait
- [ ] 🔴 Multi-gameweek projection aggregation
- [ ] 🔴 **<UNCLEAR, CLARIFY>**: Number of gameweeks to include?
- [ ] 🔴 Integration test

#### Iterative Squads (`src/problems/iterative.rs`)
- [ ] 🔴 `IterativeSquadsConfig` struct
- [ ] 🔴 Sequential solving with cutoff
- [ ] 🔴 Randomized objective weights
- [ ] 🔴 JSON output format
- [ ] 🔴 Integration test
- [ ] 🔴 Performance benchmark

---

## Phase 6: Solver Integration (Week 3-4)

### 6.1 Solver Trait (`src/solver/traits.rs`)
- [ ] 🔴 `Solver` trait definition
- [ ] 🔴 `solve()` method
- [ ] 🔴 `export_mps()` method
- [ ] 🔴 `set_time_limit()` method

### 6.2 CBC Backend (`src/solver/cbc.rs`)
- [ ] 🔴 `CbcSolver` struct
- [ ] 🔴 Implement `Solver` trait
- [ ] 🔴 Model conversion to `good_lp` format
- [ ] 🔴 Solution parsing
- [ ] 🔴 Time limit support
- [ ] 🔴 **<UNCLEAR, CLARIFY>**: Warm start support?

### 6.3 HiGHS Backend (`src/solver/highs.rs`) [Optional]
- [ ] 🔴 `HighsSolver` struct
- [ ] 🔴 Implement `Solver` trait
- [ ] 🔴 Performance comparison with CBC

### 6.4 MPS Export (`src/solver/mps.rs`)
- [ ] 🔴 MPS file generation
- [ ] 🔴 Match Python MPS format adjustments
- [ ] 🔴 Verify CBC can read exported files

### 6.5 Solution Types (`src/solver/solution.rs`)
- [ ] 🔴 `Solution` struct
- [ ] 🔴 Variable value lookup
- [ ] 🔴 Objective value getter
- [ ] 🔴 Solution status enum

---

## Phase 7: Output Formatting (Week 4)

### 7.1 CSV Output (`src/output/csv.rs`)
- [ ] 🔴 Match Python CSV format exactly
- [ ] 🔴 Column ordering
- [ ] 🔴 Calculated fields (gw_points)
- [ ] 🔴 Sorting by position/lineup status

### 7.2 JSON Output (`src/output/json.rs`)
- [ ] 🔴 Iterative results format
- [ ] 🔴 Squad/lineup/bench structure
- [ ] 🔴 Objective breakdown

### 7.3 Display Output (`src/output/display.rs`)
- [ ] 🔴 Terminal table formatting
- [ ] 🔴 Colorized output (optional)
- [ ] 🔴 Summary statistics

---

## Phase 8: CLI & Configuration (Week 4-5)

### 8.1 Configuration System (`src/config/`)
- [ ] 🔴 TOML config file support
- [ ] 🔴 `Config` struct with all parameters
- [ ] 🔴 Default configuration
- [ ] 🔴 Environment variable overrides
- [ ] 🔴 `static-values.json` compatibility

### 8.2 CLI Interface (`src/main.rs`)
- [ ] 🔴 `clap` derive setup
- [ ] 🔴 `solve` subcommand
- [ ] 🔴 `iterative` subcommand
- [ ] 🔴 `export` subcommand
- [ ] 🔴 `update` subcommand (data fetch)
- [ ] 🔴 Verbose/quiet output levels
- [ ] 🔴 Help text and examples

---

## Phase 9: Testing & Quality (Week 5)

### 9.1 Unit Tests
- [ ] 🔴 All type modules tested
- [ ] 🔴 All constraint modules tested
- [ ] 🔴 Data loading tested
- [ ] 🔴 Model builder tested

### 9.2 Integration Tests
- [ ] 🔴 End-to-end problem solving
- [ ] 🔴 CSV output verification
- [ ] 🔴 Comparison with Python results

### 9.3 Property-Based Tests
- [ ] 🔴 Constraint validity (always satisfiable)
- [ ] 🔴 Budget never exceeded
- [ ] 🔴 Position counts correct

### 9.4 Benchmarks
- [ ] 🔴 Single solve benchmark
- [ ] 🔴 Iterative solve benchmark
- [ ] 🔴 Memory usage profiling
- [ ] 🔴 Comparison with Python performance

### 9.5 Code Quality
- [ ] 🔴 No clippy warnings
- [ ] 🔴 Consistent formatting
- [ ] 🔴 Documentation for public API
- [ ] 🔴 Example code

---

## Phase 10: Documentation (Week 5-6)

### 10.1 API Documentation
- [ ] 🔴 Rustdoc for all public items
- [ ] 🔴 Module-level documentation
- [ ] 🔴 Example usage in docs

### 10.2 User Documentation
- [ ] 🔴 README with installation
- [ ] 🔴 Configuration guide
- [ ] 🔴 CLI usage guide
- [ ] 🔴 Migration from Python guide

### 10.3 Developer Documentation
- [ ] 🔴 Architecture overview
- [ ] 🔴 Adding new constraints guide
- [ ] 🔴 Adding new problems guide
- [ ] 🔴 Adding new solver backends guide

---

## Stretch Goals (Future)

### Advanced Features
- [ ] 🔴 Transfer optimization (multi-week planning)
- [ ] 🔴 Wildcard chip strategy
- [ ] 🔴 Free Hit chip strategy
- [ ] 🔴 Triple Captain chip strategy
- [ ] 🔴 Stochastic optimization
- [ ] 🔴 Risk-adjusted objectives (CVaR)

### Performance
- [ ] 🔴 Parallel iterative solving
- [ ] 🔴 Model caching
- [ ] 🔴 Incremental solving

### Integrations
- [ ] 🔴 Web API (REST)
- [ ] 🔴 WebAssembly build
- [ ] 🔴 Python bindings (PyO3)

---

## Known Issues & Blockers

### Current Blockers
1. None yet

### Known Issues
1. None yet

### Technical Debt
1. None yet

---

## Progress Tracking

| Phase | Status | Start Date | End Date | Notes |
|-------|--------|------------|----------|-------|
| Phase 1 | 🔴 Not Started | - | - | |
| Phase 2 | 🔴 Not Started | - | - | |
| Phase 3 | 🔴 Not Started | - | - | |
| Phase 4 | 🔴 Not Started | - | - | |
| Phase 5 | 🔴 Not Started | - | - | |
| Phase 6 | 🔴 Not Started | - | - | |
| Phase 7 | 🔴 Not Started | - | - | |
| Phase 8 | 🔴 Not Started | - | - | |
| Phase 9 | 🔴 Not Started | - | - | |
| Phase 10 | 🔴 Not Started | - | - | |

---

## Changelog

### 2026-01-08
- Initial TODO checklist created
- 10 phases defined
- ~150 tasks identified

---

*Last updated: 2026-01-08*
