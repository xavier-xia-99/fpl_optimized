# Design Tradeoffs & Architectural Decisions

This document captures the key design decisions for the Rust implementation, including tradeoffs considered and rationale for choices made.

---

## Table of Contents

1. [Solver Backend Selection](#1-solver-backend-selection)
2. [Type System Design](#2-type-system-design)
3. [Constraint Architecture](#3-constraint-architecture)
4. [Data Storage Strategy](#4-data-storage-strategy)
5. [Error Handling Strategy](#5-error-handling-strategy)
6. [Parallelization Approach](#6-parallelization-approach)
7. [API Design Decisions](#7-api-design-decisions)
8. [Testing Strategy](#8-testing-strategy)

---

## 1. Solver Backend Selection

### Decision: Use `good_lp` with CBC as default backend

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **A: `good_lp` + CBC** | Solver-agnostic, easy to switch | Extra abstraction layer |
| **B: Direct `coin_cbc`** | Minimal overhead, direct control | Locked to CBC |
| **C: `highs` crate** | Modern solver, MIT license | Different solver = different results |
| **D: Custom FFI** | Full control | Significant development effort |

### Rationale

**Selected: Option A (`good_lp` + CBC)**

1. **Portability**: Easy to add HiGHS or Gurobi support later
2. **Consistency**: CBC matches Python implementation
3. **Maintenance**: `good_lp` is actively maintained
4. **Feature Coverage**: Sufficient for our problem sizes

### Trade-off Impact

- **Performance**: ~5-10% overhead from abstraction (acceptable)
- **Flexibility**: Can swap solvers without code changes
- **Compatibility**: Same solutions as Python (important for validation)

### **<UNCLEAR, CLARIFY>**: Commercial Solver Support

Do we need native Gurobi/CPLEX support? If so:
- `good_lp` doesn't support Gurobi directly
- Would need custom integration or different approach

---

## 2. Type System Design

### Decision: Use newtypes for domain primitives

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **A: Raw primitives** | Simple, no boilerplate | Type confusion bugs |
| **B: Newtypes** | Type safety, semantic clarity | More boilerplate |
| **C: Type aliases** | Documentation value | No compile-time safety |

### Rationale

**Selected: Option B (Newtypes)**

```rust
// Instead of:
fn get_player(id: u32) -> Player;

// We use:
fn get_player(id: PlayerId) -> Player;
```

Benefits:
1. Cannot accidentally pass `TeamCode` where `PlayerId` expected
2. Methods like `Cost::to_millions()` provide domain logic
3. Self-documenting code
4. Derive traits for serialization

### Trade-off Impact

- **Code Volume**: ~20% more boilerplate
- **Safety**: Eliminates entire class of bugs
- **Performance**: Zero runtime cost (newtypes are erased)

### Implementation Pattern

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u32);

impl PlayerId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

---

## 3. Constraint Architecture

### Decision: Trait-based constraint composition

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **A: Monolithic functions** | Simple, mirrors Python | Hard to compose/test |
| **B: Trait-based** | Composable, testable | More complex |
| **C: Macro-based DSL** | Concise constraint specs | Learning curve, debugging |

### Rationale

**Selected: Option B (Trait-based)**

```rust
pub trait Constraint {
    fn name(&self) -> &'static str;
    fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) -> Result<(), ConstraintError>;
    fn is_required(&self) -> bool { true }
}
```

Benefits:
1. Each constraint is independently testable
2. Problems are assembled from constraint collections
3. Easy to add new constraints without modifying existing code
4. Constraints can be enabled/disabled at runtime

### Trade-off Impact

- **Complexity**: Slightly higher initial complexity
- **Flexibility**: Problems can be defined declaratively
- **Testing**: Each constraint unit-testable

### Pattern Example

```rust
impl Problem for LimitedBestSquad {
    fn constraints(&self) -> Vec<Box<dyn Constraint>> {
        vec![
            Box::new(SquadSizeConstraint::default()),
            Box::new(BudgetConstraint { budget: self.budget }),
            Box::new(TeamLimitConstraint { max_per_team: 3 }),
            // ... more constraints
        ]
    }
}
```

---

## 4. Data Storage Strategy

### Decision: HashMap with tuple keys for multi-indexed data

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **A: Nested HashMaps** | Hierarchical access | Two lookups, awkward iteration |
| **B: Tuple keys** | O(1) lookup, flat structure | Slightly more memory |
| **C: Custom struct** | Type-safe indexing | Implementation effort |
| **D: ndarray** | Fast numerics | Overkill for our needs |

### Rationale

**Selected: Option B (Tuple keys)**

```rust
// Player-Gameweek projections
type ProjectionMap = HashMap<(PlayerId, Gameweek), PlayerGameweekProjection>;

// Access:
let proj = map.get(&(player_id, gameweek));
```

Benefits:
1. Single lookup for any (player, gameweek) pair
2. Easy iteration with `.iter()`
3. Memory-efficient for sparse data
4. Simple implementation

### Trade-off Impact

- **Memory**: Hash overhead, but negligible for ~600 players × 3 GWs
- **Speed**: O(1) lookups
- **Simplicity**: Straightforward implementation

### **<UNCLEAR, CLARIFY>**: Large Horizon Scaling

For `SetAndForget` with 38 gameweeks × 600 players = 22,800 entries.
HashMap should still be fine, but consider:
- Pre-allocation with `HashMap::with_capacity()`
- Using `rustc-hash` for faster hashing

---

## 5. Error Handling Strategy

### Decision: `thiserror` for library, `anyhow` for application

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **A: String errors** | Simple | No structure, hard to handle |
| **B: `thiserror` only** | Structured, type-safe | Verbose at boundaries |
| **C: `anyhow` only** | Convenient, context | Less structured |
| **D: Hybrid** | Best of both | Two error systems |

### Rationale

**Selected: Option D (Hybrid)**

```rust
// Library errors (src/error.rs)
#[derive(thiserror::Error, Debug)]
pub enum SolverError {
    #[error("Model is infeasible")]
    Infeasible,
    #[error("Solver timeout after {0} seconds")]
    Timeout(u64),
    #[error("Variable '{0}' not found")]
    VariableNotFound(String),
}

// Application boundary (src/main.rs)
fn main() -> anyhow::Result<()> {
    let result = solve().context("Failed to solve problem")?;
    Ok(())
}
```

Benefits:
1. Library consumers get typed, matchable errors
2. Application gets convenient error handling with context
3. Clear separation of concerns

### Trade-off Impact

- **Complexity**: Two error types to understand
- **Usability**: Good ergonomics for both library and app users
- **Debugging**: Rich error context

---

## 6. Parallelization Approach

### Decision: Sequential by default, opt-in parallelism

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **A: Always parallel** | Max performance | Complexity, thread safety |
| **B: Sequential only** | Simple, matches Python | Slower for iterative |
| **C: Opt-in parallel** | User choice | Configuration complexity |

### Rationale

**Selected: Option C (Opt-in parallel)**

The iterative problem has data dependencies (cutoff from previous solution), 
making full parallelism non-trivial.

```rust
impl IterativeSquads {
    // Sequential (matches Python behavior)
    pub fn solve_sequential(&self, solver: &impl Solver, data: &ProblemData) -> Result<Vec<ProblemResult>> {
        // Each iteration depends on previous
    }
    
    // Parallel variant (different algorithm)
    pub fn solve_parallel(&self, solver: &impl Solver + Sync, data: &ProblemData) -> Result<Vec<ProblemResult>> {
        // Independent runs with different random seeds
    }
}
```

### Trade-off Impact

- **Compatibility**: Sequential matches Python exactly
- **Performance**: Parallel can be ~4-8× faster on multi-core
- **Determinism**: Sequential is reproducible; parallel may vary

### **<UNCLEAR, CLARIFY>**: Parallel Algorithm

For parallel iterative solving, options are:
1. **Independent Seeds**: Run N independent optimizations
2. **Parallel Diversification**: Solve base problem in parallel with different objectives
3. **Portfolio Solving**: Try multiple solver strategies simultaneously

Current recommendation: Start with sequential, add parallel as optimization.

---

## 7. API Design Decisions

### Decision: Builder pattern for model construction

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **A: Direct struct construction** | Simple | Verbose, error-prone |
| **B: Builder pattern** | Validated, fluent | More code |
| **C: Macro-based DSL** | Concise | Magic, hard to debug |

### Rationale

**Selected: Option B (Builder pattern)**

```rust
let model = ModelBuilder::new()
    .init_variables(&players)
    .add_constraint(SquadSizeConstraint::default())
    .add_constraint(BudgetConstraint { budget })
    .set_objective(Objective::MaximizeExpectedPoints)
    .build()?;
```

Benefits:
1. Compile-time checks (e.g., objective must be set before build)
2. Chainable, readable API
3. Can validate during construction
4. Impossible to create invalid models

### Trade-off Impact

- **Ergonomics**: Excellent user experience
- **Code Volume**: More implementation code
- **Validation**: Errors caught at build time

### Typestate Pattern (Advanced)

Consider using typestate to ensure correct build order:

```rust
struct ModelBuilder<State> { ... }

impl ModelBuilder<NoVariables> {
    fn init_variables(self, players: &[Player]) -> ModelBuilder<HasVariables> { ... }
}

impl ModelBuilder<HasVariables> {
    fn add_constraint(self, c: impl Constraint) -> Self { ... }
    fn set_objective(self, obj: Objective) -> ModelBuilder<HasObjective> { ... }
}

impl ModelBuilder<HasObjective> {
    fn build(self) -> Model { ... }  // Only callable with objective set
}
```

---

## 8. Testing Strategy

### Decision: Three-tier testing approach

### Testing Tiers

| Tier | Type | Purpose | Coverage |
|------|------|---------|----------|
| 1 | Unit Tests | Individual components | Types, constraints |
| 2 | Integration Tests | End-to-end flows | Problems, solving |
| 3 | Property Tests | Invariant verification | Constraint validity |

### Rationale

1. **Unit tests** catch regressions in individual components
2. **Integration tests** verify system behavior matches Python
3. **Property tests** ensure constraints are never violated

### Example Property Test

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn solution_always_respects_budget(
        budget in 800u16..1200,
        seed in any::<u64>()
    ) {
        let config = Config { budget: Cost(budget), ..Default::default() };
        let data = generate_test_data(seed);
        let result = solve_limited_best_squad(&config, &data)?;
        
        let total_cost: u16 = result.squad.iter()
            .map(|id| data.players[id].cost.0)
            .sum();
        
        prop_assert!(total_cost <= budget);
    }
}
```

### Trade-off Impact

- **Coverage**: High confidence in correctness
- **Time**: Property tests can be slow
- **Maintenance**: Tests need updating with API changes

---

## Summary of Key Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| Solver | `good_lp` + CBC | Portable, matches Python |
| Types | Newtypes | Type safety, zero cost |
| Constraints | Trait-based | Composable, testable |
| Data | Tuple-keyed HashMap | O(1) lookup, simple |
| Errors | thiserror + anyhow | Structured + convenient |
| Parallelism | Opt-in | Compatibility + performance |
| API | Builder pattern | Validated, fluent |
| Testing | Three-tier | Comprehensive coverage |

---

## Open Questions

### **<UNCLEAR, CLARIFY>**: MPS Compatibility

The Python code adjusts MPS format for CBC compatibility:
- Field positioning
- Keyword spacing

Need to verify if `good_lp`'s MPS export is compatible or if we need similar adjustments.

### **<UNCLEAR, CLARIFY>**: Warm Starting

Does CBC accept warm starts? If so:
- Can significantly speed up iterative solving
- Need to implement initial solution injection

### **<UNCLEAR, CLARIFY>**: Memory Budget

For very large iterative runs (100+ solutions):
- Should we stream results to disk?
- Memory-map large data structures?
- Current estimate: ~50MB for 50 solutions (acceptable)

---

## Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2026-01-08 | 1.0 | Initial document |

---

*Last Updated: 2026-01-08*
