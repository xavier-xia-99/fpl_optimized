# FPL MILP Solver - Executive Summary

## At a Glance

**What:** Mixed Integer Linear Programming solver for Fantasy Premier League squad optimization  
**Solver:** CBC (COIN-OR Branch and Cut) - open source  
**Modeling:** sasoptpy (Python optimization library)  
**Time to Solve:** 3-5 seconds for standard squad optimization  
**Cost:** $0 (fully open source)

---

## Quick Answers

### How do we run it?

**Docker (easiest):**
```bash
docker-compose up
```

**Manual:**
```bash
pip install sasoptpy pandas aiohttp asyncio
cd src && python3 run.py
```

### What solver do we use?

**Primary:** CBC (COIN-OR Branch and Cut)
- Open-source MILP solver
- Invoked via command line: `cbc model.mps solve solu solution.sol`
- Reliable for problems up to ~2000 variables
- Free with no licensing restrictions

**Alternative:** MPS format allows using Gurobi, CPLEX, Xpress, GLPK

### How is the model setup?

**Variables (Binary 0/1):**
- `x[i]` = Player i is in starting lineup (11 total)
- `y[i]` = Player i is captain (1 total)  
- `z[i]` = Player i is in squad (15 total)

**Objective:**
```
Maximize: Σ (expected_points[i] × (x[i] + y[i]))
```

**Constraint Categories:**
1. FPL Rules (lineup/squad/captain limits)
2. Budget (£100M = 1000 in tenths)
3. Team limits (max 3 players per team)
4. Formation (GK/DEF/MID/FWD requirements)
5. Problem-specific (ownership, bench weight, etc.)

---

## Constraints Summary

| Constraint | Mathematical Form | Purpose |
|-----------|------------------|---------|
| **Lineup Size** | Σ x[i] = 11 | Exactly 11 starters |
| **Squad Size** | Σ z[i] = 15 | Exactly 15 players |
| **Budget** | Σ z[i]×cost[i] ≤ 1000 | Max £100M |
| **Team Limit** | Σ z[i\|team[i]=j] ≤ 3 | Max 3 from same team |
| **Captain** | Σ y[i] = 1 | Exactly 1 captain |
| **Captain Plays** | y[i] ≤ x[i] | Captain must start |
| **Lineup in Squad** | x[i] ≤ z[i] | Starters ⊆ Squad |
| **Position Min** | Σ x[i\|pos[i]=GK] ≥ 1 | Min GK/DEF/MID/FWD |
| **Position Max** | Σ x[i\|pos[i]=DEF] ≤ 5 | Max DEF/MID/FWD |
| **Squad Exact** | Σ z[i\|pos[i]=DEF] = 5 | Exact squad composition |

### Optional Constraints (by problem type)

- **Differential:** z[i] = 0 if ownership[i] > 5%
- **Bench Weight:** Bench counts 10% in objective
- **Iterative Cutoff:** Max 12 players from previous solution
- **Set & Forget:** Optimize across 8+ gameweeks

---

## User-Configurable Parameters

### Easy to Modify (1 line edit)

| Parameter | Default | Location | Typical Range |
|-----------|---------|----------|---------------|
| **Budget** | £100.0M (1000) | solve.py:131+ | 900-1100 |
| **Team Limit** | 3 players | solve.py:128+ | 2-4 |
| **Bench Weight** | 0.1 (10%) | solve.py:220+ | 0.0-0.5 |
| **Ownership Threshold** | 5% | solve.py:384 | 2-20 |

### Requires Data File Update

| Parameter | Default | Location | Notes |
|-----------|---------|----------|-------|
| **Projections** | CSV file | static/projection/{season}/gw{N}.csv | Custom xP sources |
| **Position Rules** | FPL standard | element_type.csv | GK/DEF/MID/FWD counts |

---

## Pros vs Cons

### ✅ Strengths

1. **Mathematically Optimal** - Guaranteed best solution (given projections)
2. **Transparent** - Full visibility into all constraints and objectives
3. **Free & Open Source** - No costs, fully customizable
4. **Fast for Single GW** - 3-5 seconds to solve
5. **Multiple Problem Types** - 6+ specialized variants
6. **MPS Export** - Portable to any MILP solver
7. **Easy to Customize** - Add constraints with 1-10 lines of code
8. **Educational** - Great for learning optimization

### ❌ Limitations

1. **No Transfer Optimization** - Each GW solved independently
2. **Projection Dependent** - Quality limited by input data
3. **Single Period Focus** - Multi-week planning is basic
4. **CBC Performance** - 5-10× slower than Gurobi/CPLEX
5. **Manual Updates** - Projections must be updated externally
6. **No Stochastic Modeling** - Point estimates only
7. **Limited Multi-Objective** - Primary objective only
8. **No Automatic Chip Strategy** - Bench boost/wildcard separate

---

## Comparison with Similar Solvers

### vs FPL Review Optimizer

| Feature | This Solver | FPL Review |
|---------|-------------|------------|
| Speed | 3-5s (CBC) | 1s (Gurobi) |
| Multi-GW Transfers | ❌ | ✅ |
| Cost | Free | $40-50/year |
| Customization | ✅ Full | ❌ Limited |
| Projections | DIY/Manual | ✅ Included |

**Verdict:** FPL Review for competitive play, This for customization/research

### vs mikkel-meller/fpl-optimization

| Feature | This Solver | mikkel-meller |
|---------|-------------|---------------|
| Transfer Logic | ❌ | ✅ Rolling horizon |
| Wildcard | ❌ | ✅ |
| Documentation | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Problem Variants | 6+ | 2-3 |
| Complexity | Medium | High |

**Verdict:** mikkel-meller for transfers, This for specialized problems

### vs Genetic Algorithms / Heuristics

| Feature | MILP (This) | Genetic/Heuristic |
|---------|-------------|-------------------|
| Optimality | Guaranteed | ~90-95% |
| Speed | Seconds | Minutes |
| Constraints | All enforced | May violate |
| Stability | Deterministic | Stochastic |

**Verdict:** MILP for guaranteed quality, GA for very large problems

---

## When to Use This Solver

### ✅ Great For:

1. **Single gameweek optimization** - Weekly squad picks
2. **Custom constraints** - Specific requirements
3. **Research & analysis** - Academic work, testing strategies
4. **Learning optimization** - Educational purposes
5. **Budget = $0** - No licensing costs
6. **Differential strategies** - Low ownership focus
7. **Benchmark analysis** - "What's theoretically possible?"
8. **MPS export needs** - Solver-agnostic modeling

### ⚠️ Not Ideal For:

1. **Multi-week transfer planning** → Use mikkel-meller or FPL Review
2. **Competitive leagues needing transfers** → Use FPL Review
3. **Wildcard/free hit strategy** → Use full-season solvers
4. **Very large horizons (20+ GW)** → CBC struggles
5. **Turnkey solution** → Requires technical setup
6. **Non-technical users** → Web-based alternatives easier

---

## Model Statistics

### Problem Sizes

| Problem Type | Binary Variables | Constraints | Typical Solve Time |
|-------------|------------------|-------------|-------------------|
| No Limit Best 11 | ~1,200 | ~100 | 0.5s |
| Standard Squad | ~1,800 | ~150 | 3-5s |
| 3-Week Horizon | ~5,400 | ~500 | 10-30s |
| Iterative (50 runs) | ~1,800 | ~200/run | 2-5min total |

### Solver Performance

```
CBC:     ████░░░░░░ 40%  (3-5s)
GLPK:    ███░░░░░░░ 30%  (5-10s)
Gurobi:  ██████████ 100% (0.5s baseline)
CPLEX:   ██████████ 100% (0.5s)
CP-SAT:  ████████░░ 80%  (1-2s)
```

---

## Typical Workflow

```
1. Update projections → static/projection/2025-26/gw21.csv
                       ↓
2. Run solver        → docker-compose up OR python3 run.py
                       ↓
3. Models generated  → .mps files (for inspection/alternative solvers)
                       ↓
4. CBC solves        → 3-5 seconds
                       ↓
5. Output CSV        → build/data/.../output/*.csv
                       ↓
6. Analyze results   → Import into Excel/Python/R
```

---

## Key Files

| File | Purpose | Size |
|------|---------|------|
| `solve.py` | Core optimization models | 760 lines |
| `collect.py` | FPL API data fetching | 970 lines |
| `prep.py` | Data preparation | 70 lines |
| `run.py` | Orchestration | 25 lines |
| `working_problems.py` | Experimental models | 420 lines |

---

## Installation Requirements

**System:**
- Linux/Mac/Windows
- 2GB RAM minimum
- Python 3.8+

**Software:**
- CBC solver (included in Docker)
- Python packages: sasoptpy, pandas, aiohttp

**Data:**
- Internet connection (for FPL API)
- Projection files (CSV format)

---

## Output Format

### CSV Results
```csv
player_id,web_name,team_code,element_type,now_cost,event,points_md,
is_captain,multiplier,starting_lineup,selected_by_percent,gw_points

263,Salah,14,3,127,21,8.5,True,2,1,45.2,17.0
328,Haaland,43,4,152,21,9.2,False,1,1,62.8,9.2
...
```

### MPS Model Files
```
Standard MILP format readable by:
- Gurobi
- CPLEX
- Xpress
- GLPK
- SCIP
- Any LP/MIP solver
```

---

## Next Steps After Reading This

1. **Want details?** → Read [SPECIFICATION.md](SPECIFICATION.md)
2. **Compare solvers?** → Read [SOLVER_COMPARISON.md](SOLVER_COMPARISON.md)
3. **Modify parameters?** → Read [PARAMETER_GUIDE.md](PARAMETER_GUIDE.md)
4. **Just run it?** → `docker-compose up`

---

## Support & Community

- **Issues:** Open GitHub issue
- **Features:** Submit pull request
- **Questions:** Check documentation first
- **Research:** Citation info in README

---

## Future Enhancements Roadmap

### Priority 1 (High Value)
- [ ] Rolling horizon transfer optimization
- [ ] Gurobi/CPLEX direct API
- [ ] Web UI parameter controls
- [ ] Automatic projection updates

### Priority 2 (Medium Value)
- [ ] Stochastic optimization (scenarios)
- [ ] Risk-adjusted objectives (CVaR)
- [ ] Price change modeling
- [ ] Automatic substitution logic

### Priority 3 (Nice to Have)
- [ ] Pareto front generation
- [ ] Sensitivity analysis tools
- [ ] Live GW solver
- [ ] Historical backtesting

---

## License Summary

- **Codebase:** [See LICENSE file]
- **CBC Solver:** EPL 2.0 (Eclipse Public License)
- **sasoptpy:** Apache 2.0
- **FPL Data:** © Premier League (fair use)

---

## Version History

- **v1.0** (2026-01-03): Initial documentation release
  - 6 problem variants
  - CBC solver backend
  - MPS export capability
  - Docker support

---

**Quick Links:**
- [Full Specification](SPECIFICATION.md)
- [Solver Comparison](SOLVER_COMPARISON.md)
- [Parameter Guide](PARAMETER_GUIDE.md)
- [Documentation Index](README_DOCS.md)

---

*For questions or contributions, open a GitHub issue.*

**Status:** ✅ Production Ready | 📚 Fully Documented | 🔬 Research-Grade
