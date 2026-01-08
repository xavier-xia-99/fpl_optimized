# FPL Optimization - Constraints Reference

This document provides detailed mathematical specifications for all constraints used in the FPL optimization solver.

---

## Table of Contents

1. [Variable Definitions](#variable-definitions)
2. [Hard Constraints (Always Enforced)](#hard-constraints)
3. [Problem-Specific Constraints](#problem-specific-constraints)
4. [Multi-Period Constraints](#multi-period-constraints)
5. [Implementation Notes](#implementation-notes)

---

## Variable Definitions

### Primary Decision Variables

| Variable | Type | Domain | Description |
|----------|------|--------|-------------|
| `x[i]` | Binary | {0, 1} | Player `i` is in the starting lineup |
| `y[i]` | Binary | {0, 1} | Player `i` is the captain |
| `z[i]` | Binary | {0, 1} | Player `i` is in the squad (15 players) |

### Multi-Period Variables (Iterative Problem)

| Variable | Type | Domain | Description |
|----------|------|--------|-------------|
| `lineup[i,g]` | Binary | {0, 1} | Player `i` starts in gameweek `g` |
| `captain[i,g]` | Binary | {0, 1} | Player `i` is captain in gameweek `g` |
| `squad[i]` | Binary | {0, 1} | Player `i` is in the (fixed) squad |
| `bench[i,g,p]` | Binary | {0, 1} | Player `i` is bench position `p` in gameweek `g` |

### Auxiliary Variables

| Variable | Type | Description |
|----------|------|-------------|
| `w` | Continuous | Auxiliary for max-min problems |

---

## Hard Constraints

These constraints are enforced in **all** problem variants.

### 1. Squad Size Constraint

**Mathematical Form:**
```
Σᵢ z[i] = 15
```

**Plain English:** The squad must contain exactly 15 players.

**Rust Implementation:**
```rust
fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
    let expr = data.players.iter()
        .map(|p| builder.squad_var(p.id))
        .sum::<Expression>();
    builder.add_constraint(expr.eq(15.0), "squad_limit");
}
```

---

### 2. Lineup Size Constraint

**Mathematical Form:**
```
Σᵢ x[i] = 11
```

**Plain English:** The starting lineup must contain exactly 11 players.

**Rust Implementation:**
```rust
fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
    let expr = data.players.iter()
        .map(|p| builder.lineup_var(p.id))
        .sum::<Expression>();
    builder.add_constraint(expr.eq(11.0), "lineup_limit");
}
```

---

### 3. Budget Constraint

**Mathematical Form:**
```
Σᵢ z[i] × cost[i] ≤ B

where B = 1000 (default, representing £100.0M)
```

**Plain English:** The total cost of all squad players cannot exceed the budget.

**Parameters:**
- `B` (budget): Default 1000 (tenths of millions)

**Rust Implementation:**
```rust
fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
    let expr = data.players.iter()
        .map(|p| builder.squad_var(p.id) * p.cost.0 as f64)
        .sum::<Expression>();
    builder.add_constraint(expr.leq(self.budget.0 as f64), "total_cost");
}
```

---

### 4. Team Limit Constraint

**Mathematical Form:**
```
∀j ∈ Teams: Σᵢ∈T(j) z[i] ≤ L

where T(j) = {players belonging to team j}
      L = 3 (default)
```

**Plain English:** No more than 3 players from the same Premier League team.

**Parameters:**
- `L` (limit): Default 3

**Rust Implementation:**
```rust
fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
    for team in data.team_codes {
        let expr = data.players.iter()
            .filter(|p| p.team_code == *team)
            .map(|p| builder.squad_var(p.id))
            .sum::<Expression>();
        builder.add_constraint(
            expr.leq(self.limit as f64),
            &format!("team_limit_{}", team.0)
        );
    }
}
```

---

### 5. Squad Position Exact Constraint

**Mathematical Form:**
```
∀p ∈ Positions: Σᵢ∈P(p) z[i] = select(p)

where P(p) = {players with position p}
      select(GK) = 2
      select(DEF) = 5
      select(MID) = 5
      select(FWD) = 3
```

**Plain English:** Squad must have exactly 2 GK, 5 DEF, 5 MID, 3 FWD.

**Position Requirements Table:**

| Position | squad_select |
|----------|--------------|
| GK       | 2            |
| DEF      | 5            |
| MID      | 5            |
| FWD      | 3            |
| **Total**| **15**       |

**Rust Implementation:**
```rust
fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
    for pos in Position::all() {
        let expr = data.players.iter()
            .filter(|p| p.position == pos)
            .map(|p| builder.squad_var(p.id))
            .sum::<Expression>();
        builder.add_constraint(
            expr.eq(pos.squad_select() as f64),
            &format!("squad_exact_{:?}", pos)
        );
    }
}
```

---

### 6. Lineup Position Min/Max Constraints

**Mathematical Form:**
```
∀p ∈ Positions: min_play(p) ≤ Σᵢ∈P(p) x[i] ≤ max_play(p)

where min_play(GK) = 1,  max_play(GK) = 1
      min_play(DEF) = 3, max_play(DEF) = 5
      min_play(MID) = 2, max_play(MID) = 5
      min_play(FWD) = 1, max_play(FWD) = 3
```

**Plain English:** Lineup must respect formation rules (e.g., 1 GK, 3-5 DEF, 2-5 MID, 1-3 FWD).

**Position Limits Table:**

| Position | min_play | max_play |
|----------|----------|----------|
| GK       | 1        | 1        |
| DEF      | 3        | 5        |
| MID      | 2        | 5        |
| FWD      | 1        | 3        |
| **Total**| **7**    | **14**   |

**Valid Formations (11 players):**
- 4-4-2, 4-3-3, 4-5-1
- 3-4-3, 3-5-2
- 5-3-2, 5-4-1
- etc.

**Rust Implementation:**
```rust
fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
    for pos in Position::all() {
        let expr = data.players.iter()
            .filter(|p| p.position == pos)
            .map(|p| builder.lineup_var(p.id))
            .sum::<Expression>();
        
        // Min constraint
        builder.add_constraint(
            expr.clone().geq(pos.min_play() as f64),
            &format!("lineup_min_{:?}", pos)
        );
        
        // Max constraint
        builder.add_constraint(
            expr.leq(pos.max_play() as f64),
            &format!("lineup_max_{:?}", pos)
        );
    }
}
```

---

### 7. Captain Constraint

**Mathematical Form:**
```
Σᵢ y[i] = 1
```

**Plain English:** Exactly one player must be captain.

**Rust Implementation:**
```rust
fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
    let expr = data.players.iter()
        .map(|p| builder.captain_var(p.id))
        .sum::<Expression>();
    builder.add_constraint(expr.eq(1.0), "single_captain");
}
```

---

### 8. Captain Must Play Constraint

**Mathematical Form:**
```
∀i: y[i] ≤ x[i]
```

**Plain English:** The captain must be in the starting lineup.

**Rust Implementation:**
```rust
fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
    for player in data.players {
        let captain = builder.captain_var(player.id);
        let lineup = builder.lineup_var(player.id);
        builder.add_constraint(
            captain.leq(lineup),
            &format!("captain_plays_{}", player.id.0)
        );
    }
}
```

---

### 9. Lineup Subset of Squad Constraint

**Mathematical Form:**
```
∀i: x[i] ≤ z[i]
```

**Plain English:** Every player in the lineup must also be in the squad.

**Rust Implementation:**
```rust
fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
    for player in data.players {
        let lineup = builder.lineup_var(player.id);
        let squad = builder.squad_var(player.id);
        builder.add_constraint(
            lineup.leq(squad),
            &format!("lineup_in_squad_{}", player.id.0)
        );
    }
}
```

---

## Problem-Specific Constraints

### 10. Differential Ownership Constraint

**Used In:** `DifferentialSquad` problem

**Mathematical Form:**
```
∀i where ownership[i] > T: z[i] = 0

where T = 5.0% (default threshold)
```

**Plain English:** Players owned by more than 5% of managers cannot be selected.

**Parameters:**
- `T` (threshold): Default 5.0%

**Rust Implementation:**
```rust
fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
    for player in data.players {
        if player.ownership_percent > self.threshold {
            builder.add_constraint(
                builder.squad_var(player.id).eq(0.0),
                &format!("diff_ban_{}", player.id.0)
            );
        }
    }
}
```

---

### 11. Iterative Squad Cutoff Constraint

**Used In:** `IterativeSquads` problem

**Mathematical Form:**
```
For iteration k > 0:
Σᵢ∈S(k-1) z[i] ≤ C

where S(k-1) = squad selected in iteration k-1
      C = 12 (default cutoff)
```

**Plain English:** At most 12 players can overlap with the previous solution.

**Parameters:**
- `C` (cutoff): Default 12

**Rust Implementation:**
```rust
fn apply(&self, builder: &mut ModelBuilder, prev_squad: &[PlayerId]) {
    let expr = prev_squad.iter()
        .map(|id| builder.squad_var(*id))
        .sum::<Expression>();
    builder.add_constraint(
        expr.leq(self.cutoff as f64),
        &format!("cutoff_{}", self.iteration)
    );
}
```

---

## Multi-Period Constraints

### 12. Lineup Per Gameweek Constraint

**Used In:** `IterativeSquads`, `SetAndForget`

**Mathematical Form:**
```
∀g ∈ Gameweeks: Σᵢ lineup[i,g] = 11
```

**Plain English:** Each gameweek must have exactly 11 starters.

---

### 13. Captain Per Gameweek Constraint

**Mathematical Form:**
```
∀g ∈ Gameweeks: Σᵢ captain[i,g] = 1
```

**Plain English:** Each gameweek must have exactly one captain.

---

### 14. Captain Must Play Per Gameweek

**Mathematical Form:**
```
∀i, ∀g: captain[i,g] ≤ lineup[i,g]
```

---

### 15. Lineup Subset of Fixed Squad

**Mathematical Form:**
```
∀i, ∀g: lineup[i,g] ≤ squad[i]
```

**Plain English:** Can only start players from the fixed squad.

---

### 16. Bench Position Constraints

**Mathematical Form:**
```
∀g, ∀p: Σᵢ bench[i,g,p] = 1           (each position filled once)
∀g: Σᵢ bench[i,g,1] where pos[i]=GK = 1  (position 1 is GK)
∀i, ∀g: lineup[i,g] + Σₚ bench[i,g,p] = squad[i]  (lineup + bench = squad)
```

**Plain English:** 
- Each bench position has exactly one player
- Bench position 1 must be a goalkeeper
- Players are either in lineup or on bench (if in squad)

---

### 17. Play At Least Once Constraint

**Mathematical Form:**
```
∀i: Σg lineup[i,g] ≥ squad[i]
```

**Plain English:** Every squad player must start at least once in the horizon.

**<UNCLEAR, CLARIFY>**: Is this constraint actually in the Python implementation?

---

## Implementation Notes

### Constraint Ordering

Constraints should be added in this order for solver efficiency:

1. Equality constraints first (squad_size, lineup_size, squad_exact)
2. Simple inequality constraints (budget, team_limit)
3. Per-player linking constraints (captain_plays, lineup_in_squad)
4. Problem-specific constraints

### Constraint Naming Convention

Use consistent naming for debugging:
- `{constraint_type}_{identifier}`
- Examples: `squad_limit`, `team_limit_14`, `lineup_min_DEF`

### Handling Infeasibility

If the problem is infeasible, check in this order:
1. Budget too restrictive?
2. Differential threshold too low?
3. Team limits conflicting with required players?
4. Position requirements impossible?

### Coefficient Scaling

For numerical stability:
- Cost coefficients are in tenths of millions (integers)
- Objective coefficients (expected points) are typically 0-20
- No scaling needed for standard FPL problems

---

## Constraint Summary Table

| # | Constraint | Variables | RHS | Type | Used In |
|---|-----------|-----------|-----|------|---------|
| 1 | Squad Size | z[i] | 15 | = | All |
| 2 | Lineup Size | x[i] | 11 | = | All |
| 3 | Budget | z[i] × cost[i] | 1000 | ≤ | Most |
| 4 | Team Limit | z[i] | 3 | ≤ | Most |
| 5 | Squad Position | z[i] | varies | = | Most |
| 6 | Lineup Position Min | x[i] | varies | ≥ | All |
| 6 | Lineup Position Max | x[i] | varies | ≤ | All |
| 7 | Single Captain | y[i] | 1 | = | All |
| 8 | Captain Plays | y[i] - x[i] | 0 | ≤ | All |
| 9 | Lineup in Squad | x[i] - z[i] | 0 | ≤ | Most |
| 10 | Differential | z[i] | 0 | = | Differential |
| 11 | Squad Cutoff | z[i] | 12 | ≤ | Iterative |

---

## References

- Python implementation: `/workspace/src/solve.py`
- Specification: `/workspace/SPECIFICATION.md`
- Parameter guide: `/workspace/PARAMETER_GUIDE.md`

---

*Last Updated: 2026-01-08*
