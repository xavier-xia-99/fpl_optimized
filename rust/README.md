# FPL Optimizer - Rust Implementation

This directory contains the specification and implementation guide for porting the FPL Optimization Solver from Python to Rust.

## 📚 Documentation Overview

| Document | Purpose | Read First? |
|----------|---------|-------------|
| [RUST_IMPLEMENTATION.md](RUST_IMPLEMENTATION.md) | Complete implementation guide with code examples | ✅ Yes |
| [TODO_CHECKLIST.md](TODO_CHECKLIST.md) | Detailed task breakdown with progress tracking | ✅ Yes |
| [CONSTRAINTS_REFERENCE.md](CONSTRAINTS_REFERENCE.md) | Mathematical specification of all constraints | Reference |
| [DESIGN_TRADEOFFS.md](DESIGN_TRADEOFFS.md) | Architectural decisions and rationale | Reference |
| [Cargo.toml](Cargo.toml) | Dependency template | When starting |

## 🚀 Quick Start

### 1. Read the Implementation Guide

Start with [RUST_IMPLEMENTATION.md](RUST_IMPLEMENTATION.md) for:
- Architecture overview
- Module structure
- Core data types
- Solver backend options
- Code examples for every component

### 2. Understand the Constraints

Review [CONSTRAINTS_REFERENCE.md](CONSTRAINTS_REFERENCE.md) for:
- Mathematical formulations
- Variable definitions
- Constraint implementations
- Edge cases

### 3. Track Progress

Use [TODO_CHECKLIST.md](TODO_CHECKLIST.md) to:
- Follow the implementation phases
- Check off completed tasks
- Identify blockers

## 🎯 Implementation Phases

| Phase | Description | Duration |
|-------|-------------|----------|
| 1 | Project Setup & Core Types | Week 1 |
| 2 | Data Loading | Week 1-2 |
| 3 | Model Builder | Week 2 |
| 4 | Constraint Implementations | Week 2-3 |
| 5 | Problem Implementations | Week 3 |
| 6 | Solver Integration | Week 3-4 |
| 7 | Output Formatting | Week 4 |
| 8 | CLI & Configuration | Week 4-5 |
| 9 | Testing & Quality | Week 5 |
| 10 | Documentation | Week 5-6 |

## ⚠️ Items Requiring Clarification

The following items are marked **<UNCLEAR, CLARIFY>** in the documentation:

### Solver Related
1. **Primary solver backend** - Should we prioritize CBC compatibility or HiGHS performance?
2. **Warm start support** - Does `coin_cbc` support warm starting?
3. **MPS format** - Do we need the same format adjustments as Python?

### Algorithm Related
4. **Parallel iterative solving** - What algorithm for parallel diversification?
5. **Multi-period indexing** - HashMap strategy confirmed?
6. **Double gameweek handling** - How should projections aggregate?

### Feature Related
7. **Commercial solver support** - Do we need Gurobi integration?
8. **Bench ordering constraint** - Is it actually used in the iterative problem?
9. **Play at least once** - Is this constraint in the Python implementation?

## 🔧 Key Technology Choices

| Area | Choice | Alternative |
|------|--------|-------------|
| Solver Backend | `good_lp` + CBC | Direct `coin_cbc` |
| Async Runtime | `tokio` | `async-std` |
| HTTP Client | `reqwest` | `hyper` |
| Serialization | `serde` | Manual parsing |
| CLI | `clap` | `structopt` |
| Error Handling | `thiserror` + `anyhow` | Custom errors |

## 📊 Expected Performance

| Problem | Python (CBC) | Rust (CBC) Target |
|---------|--------------|-------------------|
| Limited Best Squad | 3-5 sec | 1-2 sec |
| Iterative (50 squads) | 2-5 min | 30-60 sec |
| Set and Forget | 10-30 sec | 3-10 sec |

## 🧪 Testing Strategy

1. **Unit Tests**: All types and constraints
2. **Integration Tests**: Full problem solving
3. **Property Tests**: Constraint validity
4. **Benchmarks**: Performance regression

## 📁 Project Structure

```
fpl-optimizer/
├── src/
│   ├── lib.rs              # Library root
│   ├── main.rs             # CLI entry
│   ├── types/              # Domain types
│   ├── data/               # Data loading
│   ├── model/              # MILP model
│   ├── constraints/        # Constraint impls
│   ├── problems/           # Problem variants
│   ├── solver/             # Solver backends
│   ├── output/             # Result formatting
│   └── config/             # Configuration
├── tests/                  # Integration tests
├── benches/                # Benchmarks
└── Cargo.toml
```

## 🔗 Related Documentation

- [SPECIFICATION.md](../SPECIFICATION.md) - Original Python specification
- [PARAMETER_GUIDE.md](../PARAMETER_GUIDE.md) - Parameter modification guide
- [SOLVER_COMPARISON.md](../SOLVER_COMPARISON.md) - Solver comparison matrix
- [Python Implementation](../src/solve.py) - Reference implementation

## 📝 Contributing

When implementing:

1. Follow the checklist in [TODO_CHECKLIST.md](TODO_CHECKLIST.md)
2. Update progress as tasks complete
3. Add tests for all new code
4. Document any new **<UNCLEAR, CLARIFY>** items
5. Update tradeoffs document for design decisions

## 📜 License

Same as parent project - see [LICENSE](../LICENSE)

---

## Quick Reference

### Core Types
```rust
PlayerId(u32)       // Unique player identifier
TeamCode(u8)        // Premier League team code
Position            // GK, DEF, MID, FWD
Cost(u16)           // Price in tenths of millions
Gameweek(u8)        // 1-38
```

### Main Traits
```rust
Constraint          // Individual optimization constraints
Problem             // Complete optimization problems
Solver              // Solver backend abstraction
```

### Key Commands (After Implementation)
```bash
# Solve standard problem
fpl solve --gameweek 21 --problem limited

# Generate diverse squads
fpl iterative --count 50 --gameweek 21

# Export MPS model
fpl export --output model.mps

# Update data from FPL API
fpl update
```

---

*Last Updated: 2026-01-08*
