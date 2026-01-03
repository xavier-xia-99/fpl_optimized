# Quick Reference Guide - Constraints & Parameters

## How to Modify Common Parameters

### 1. Budget Adjustment

**Default:** £100.0M (1000 in tenths)

**Location:** Multiple files in `solve.py`

```python
# Lines: 131, 214, 296, 378, 467, 583
m.add_constraint(
    so.quick_sum(z[i] * next_week_df.loc[i]['now_cost'] for i in players) <= 1000,
    name='total_cost_100')
```

**To change to £105M:**
```python
<= 1050  # Instead of 1000
```

**To change to £95M:**
```python
<= 950   # Instead of 1000
```

---

### 2. Players Per Team Limit

**Default:** Maximum 3 players from same team

**Location:** Multiple files in `solve.py`

```python
# Lines: 128-129, 211-212, 293-294, 375-376, 464-465, 566-567
m.add_constraints(
    (so.quick_sum(z[i] for i in players if next_week_df.loc[i]['team_code'] == j) <= 3 
     for j in team_codes),
    name='player_team_limit')
```

**To allow 4 players per team:**
```python
<= 4  # Instead of 3
```

**To allow 2 players per team (more diverse):**
```python
<= 2  # Instead of 3
```

---

### 3. Bench Weighting

**Default:** 0.1× (10% value for bench players)

**Location:** `solve.py` lines 220, 387, 473

```python
m.set_objective(so.quick_sum(
    -next_week_df.loc[i]['points_md'] * (x[i]+y[i]+0.1*(z[i]-x[i])) for i in players
), sense='N', name='maximize_points')
```

**To give bench 20% weight:**
```python
0.2*(z[i]-x[i])  # Instead of 0.1
```

**To ignore bench completely:**
```python
# Remove the +0.1*(z[i]-x[i]) term entirely
-next_week_df.loc[i]['points_md'] * (x[i]+y[i]) for i in players
```

---

### 4. Differential Ownership Threshold

**Default:** Maximum 5% ownership

**Location:** `solve.py` line 384

```python
m.add_constraints(
    (z[i] == 0 for i in players if next_week_df.loc[i]['selected_by_percent'] > 5), 
    name='allow_only_differentials')
```

**To allow up to 10% ownership:**
```python
> 10  # Instead of > 5
```

**To allow only <2% (super differentials):**
```python
> 2   # Instead of > 5
```

---

### 5. Squad Size

**Default:** 15 players (FPL standard)

**Location:** Multiple places in `solve.py`

```python
# Lines: 126, 209, 291, 373, 462, 564
m.add_constraint(so.quick_sum(z[i] for i in players) == 15, name='squad_limit')
```

**To change to 14 players:**
```python
== 14  # Instead of 15
```

Note: This will likely require adjusting position requirements!

---

### 6. Position Requirements

**Location:** `element_type.csv` (data input)

**Current FPL Rules:**

| Position | Squad Size | Min Playing | Max Playing |
|----------|-----------|-------------|-------------|
| GK (1)   | 2         | 1           | 1           |
| DEF (2)  | 5         | 3           | 5           |
| MID (3)  | 5         | 2           | 5           |
| FWD (4)  | 3         | 1           | 3           |

**To modify:** Edit `input/element_type.csv`:

```csv
id,squad_select,squad_min_play,squad_max_play
1,2,1,1
2,5,3,5
3,5,2,5
4,3,1,3
```

**Example - Require 4 midfielders minimum:**
```csv
3,5,4,5  # Change squad_min_play from 2 to 4
```

---

### 7. Lineup Size

**Default:** 11 players (FPL standard)

**Location:** Multiple places in `solve.py`

```python
# Lines: 45, 112, 195, 277, 359, 448, 546
m.add_constraint(so.quick_sum(x[i] for i in players) == 11, name='lineup_limit')
```

**To test with 10 starters:**
```python
== 10  # Instead of 11
```

---

### 8. Captain Multiplier

**Current:** Captain gets 2× points (implicit in objective)

**Location:** `solve.py` objective functions

```python
# Captain appears once in x[i] and once in y[i] = 2× total
-next_week_df.loc[i]['points_md'] * (x[i]+y[i])
```

**To model Triple Captain (3×):**
```python
# Add a new variable tc[i] for triple captain
-next_week_df.loc[i]['points_md'] * (x[i]+y[i]+tc[i])
# Add constraint: so.quick_sum(tc[i] for i in players) == 1
```

---

### 9. Number of Iterations (Iterative Solver)

**Default:** 50 different squads

**Location:** `solve.py` line 509, function call

```python
def solve_iterative_squads(input_folder, output_folder, total_iter=50):
```

**To generate 100 solutions:**
```python
solve_iterative_squads(input_folder, output_folder, 100)
```

**To generate 10 solutions (faster):**
```python
solve_iterative_squads(input_folder, output_folder, 10)
```

---

### 10. Multi-Period Horizon

**Default:** 3 gameweeks for iterative, 8+ for set-and-forget

**Location:** `prep.py` line 21, `solve.py` line 513

```python
# prep.py
def get_multistage_data(gw=None, n=3):  # n = number of gameweeks
```

**To plan for 5 gameweeks:**
```python
def get_multistage_data(gw=None, n=5):
```

---

## Constraint Combinations

### Aggressive Differential Strategy
```python
# File: solve.py, differential team function
# Change line 384:
(z[i] == 0 for i in players if next_week_df.loc[i]['selected_by_percent'] > 2)
# Result: Only players owned by <2% of managers
```

### High Budget Team (£105M)
```python
# File: solve.py, multiple locations
<= 1050  # Budget constraint
```

### Conservative Safe Team (4 players per team allowed)
```python
# File: solve.py, team limit constraint
<= 4  # Team limit
# Allows "doubling up" on strong teams
```

### Bench-Heavy Strategy
```python
# File: solve.py, weighted bench objective
0.5*(z[i]-x[i])  # Give bench 50% weight instead of 10%
```

---

## Problem Type Selection

### Run Specific Problems

**Location:** `solve.py` function `solve_all()`

```python
def solve_all(input_folder, output_folder):
    solve_no_limit_best_11(input_folder, output_folder)           # 1
    solve_limited_best_squad(input_folder, output_folder)          # 2
    solve_limited_squad_with_bench_weight(input_folder, output_folder)  # 3
    solve_bench_boost_squad(input_folder, output_folder)           # 4
    solve_best_differential_team(input_folder, output_folder)      # 5
    solve_best_set_and_forget(input_folder, output_folder)         # 6
```

**To run only specific problems:**
```python
def solve_all(input_folder, output_folder):
    # Comment out problems you don't want
    # solve_no_limit_best_11(input_folder, output_folder)
    solve_limited_best_squad(input_folder, output_folder)  # Only this one
    # solve_bench_boost_squad(input_folder, output_folder)
```

---

## Data Source Configuration

### Expected Points Projections

**Location:** `src/collect.py` lines 176-180

```python
try:
    prediction_df = pd.read_csv(f"static/projection/{season}/gw{start_gw}.csv")
except:
    prediction_df = pd.read_csv(f"static/projection/{season}/gw{start_gw-1}.csv")
```

**To use custom projections:**

1. Create file: `static/projection/2025-26/gw{N}.csv`
2. Required columns:
   - `ID` - Player ID
   - `Name` - Player name
   - `Team` - Team name
   - `{GW}_Pts` - Expected points (e.g., `21_Pts`)
   - `{GW}_xMins` - Expected minutes (e.g., `21_xMins`)

**Format example:**
```csv
ID,Name,Team,Pos,21_Pts,21_xMins,22_Pts,22_xMins
1,Player A,ARS,DEF,6.2,85,5.8,82
2,Player B,LIV,MID,7.5,90,8.2,90
```

---

## Solver Options

### Change Solver Command

**Location:** `solve.py` lines 746-751

```python
def solve_and_get_solution(mps_file, solution_file, model, use_initial=None):
    # Current: CBC
    r = os.system(f'cbc {mps_file} solve solu {solution_file}')
```

**To use Gurobi (if installed):**
```python
r = os.system(f'gurobi_cl {mps_file} ResultFile={solution_file}')
```

**To use GLPK:**
```python
r = os.system(f'glpsol --mps {mps_file} -o {solution_file}')
```

**To add time limit (CBC):**
```python
r = os.system(f'cbc {mps_file} sec 60 solve solu {solution_file}')
# Stops after 60 seconds
```

---

## Advanced Modifications

### Add Forced Players

**Add to any solve function:**

```python
# After creating variables x, y, z
# Force specific player IDs into squad
forced_players = [263, 445, 328]  # Example player IDs

m.add_constraints(
    (z[i] == 1 for i in forced_players if i in players),
    name='forced_players')
```

### Lock Out Players

```python
# Ban specific players
banned_players = [100, 200, 300]

m.add_constraints(
    (z[i] == 0 for i in banned_players if i in players),
    name='banned_players')
```

### Minimum Total Expected Minutes

```python
# Ensure starting lineup has at least 900 minutes expected
m.add_constraint(
    so.quick_sum(next_week_df.loc[i]['xmins_md'] * x[i] for i in players) >= 900,
    name='min_minutes')
```

### Maximum Variance (Risk Control)

```python
# Add after objective
variance = so.quick_sum(
    (next_week_df.loc[i]['points_md'] - avg_points)**2 * x[i] for i in players)

m.add_constraint(variance <= max_variance, name='risk_control')
```

### Team Pairs (e.g., Liverpool attack stack)

```python
# Force at least 2 from Liverpool's attacking players
liverpool_attackers = [player_id for player_id in players 
                       if next_week_df.loc[player_id]['team_code'] == 14  # Liverpool
                       and next_week_df.loc[player_id]['element_type'] in [3, 4]]

m.add_constraint(
    so.quick_sum(z[i] for i in liverpool_attackers) >= 2,
    name='liverpool_stack')
```

---

## Output Customization

### Change Output Location

**Location:** `solve.py` line 64-66 (example)

```python
filename = str(output_folder / "no_limit_best_11.mps")
solutionname = str(output_folder / "no_limit_best_11.sol")
csvname = str(output_folder / "no_limit_best_11.csv")
```

**To change directory:**
```python
import pathlib
custom_output = pathlib.Path("/custom/path/")
csvname = str(custom_output / "no_limit_best_11.csv")
```

### Additional Output Fields

**Location:** `solve.py` line 79 (example)

```python
result_df = result_df[['player_id', 'web_name', 'team_code', 'element_type', 
                        'now_cost', 'event', 'points_md', 'is_captain', 
                        'multiplier', 'selected_by_percent']].copy()
```

**To add form, ICT, bonus:**
```python
result_df = result_df[['player_id', 'web_name', 'team_code', 'element_type', 
                        'now_cost', 'event', 'points_md', 'is_captain', 
                        'multiplier', 'selected_by_percent',
                        'form', 'ict_index', 'bonus']].copy()  # Added fields
```

---

## Season Configuration

**Location:** `static-values.json`

```json
{
    "season": "2025-26",
    "season_status": "live",
    "win_driver": "C:\\work\\chromedriver.exe",
    "unix_driver": "/usr/local/bin/chromedriver"
}
```

**To change season:**
```json
{
    "season": "2026-27",
    ...
}
```

---

## Common Modifications Checklist

| Modification | Difficulty | File | Lines |
|-------------|-----------|------|-------|
| Change budget | ⭐ Easy | solve.py | 131, 214, 296, 378, 467, 583 |
| Change team limit | ⭐ Easy | solve.py | 128, 211, 293, 375, 464, 566 |
| Change bench weight | ⭐ Easy | solve.py | 220, 387, 473 |
| Change ownership threshold | ⭐ Easy | solve.py | 384 |
| Add forced players | ⭐⭐ Medium | solve.py | After variable creation |
| Change projections source | ⭐⭐ Medium | collect.py | 176-180 |
| Switch solver | ⭐⭐⭐ Hard | solve.py | 746-751 |
| Add custom objective | ⭐⭐⭐ Hard | solve.py | Objective section |

---

## Testing Modifications

### Verify Model Syntax

```python
# After modifying solve.py
from solve import solve_limited_best_squad
from pathlib import Path

input_folder = Path("build/data/2025-26/GW21/2026-01-03/input/")
output_folder = Path("build/data/2025-26/GW21/2026-01-03/output/")

solve_limited_best_squad(input_folder, output_folder)
```

### Check MPS File

```bash
# Inspect generated model
cbc build/data/.../output/limited_best_15.mps -import -solution
```

---

## Troubleshooting

### Problem: Infeasible Solution

**Possible causes:**
- Budget too low
- Team limit too restrictive
- Position requirements impossible
- Forced players conflict with constraints

**Fix:** Relax constraints incrementally

### Problem: Slow Solving

**Possible causes:**
- Multi-period model too large
- CBC performance limits
- Poor formulation

**Fix:**
- Reduce horizon length
- Use commercial solver (Gurobi)
- Add cuts/heuristics

### Problem: Unexpected Squad

**Possible causes:**
- Projections outdated
- Constraints not as intended
- Objective mismatch

**Fix:**
- Verify input CSVs
- Print constraint violations
- Add debug logging

---

*Last Updated: 2026-01-03*
