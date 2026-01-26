# E5: Create National Round Progression Maps

**Status**: ✅ COMPLETED
**Priority**: Medium
**Estimated Complexity**: Medium
**Created**: January 2026
**Completed**: January 2026
**Commits**: [9bb05e1](https://github.com/giodl_microsoft/redistricting/commit/9bb05e1060041db5b8aeabb601543877124e54d9)
**Size**: S - 406 lines changed (5 files)

**Completion Date:** January 12, 2026
**Implementation:** Created `scripts/pipeline/create_us_national_rounds_progression.py`, integrated into pipeline, added USA Rounds tab to dashboard.

### Goal
Create national-level visualization of recursive bisection progression showing rounds 1-6+ across all states.

### Description
Generate a series of national maps showing how the US is progressively divided:
- Round 1: 2 regions (first bisection)
- Round 2: 4 regions (second bisection)
- Round 3: 8 regions
- Round 4: 16 regions
- Round 5: 32 regions
- Round 6: 64 regions
- Continue through later rounds as states complete their divisions

### Implementation Plan

#### Data Collection Phase
- Aggregate round data from all 50 states' `rounds/round_N_assignments.pkl` files
- Track which states have completed which rounds
- Handle states with different final round counts (1-district states vs 52-district California)

#### Visualization Script
**Create:** `scripts/pipeline/create_us_national_rounds_progression.py`

##### Outputs
- `us_national_round_1_2020.png` - 2 regions
- `us_national_round_2_2020.png` - 4 regions
- `us_national_round_3_2020.png` - 8 regions
- `us_national_round_4_2020.png` - 16 regions
- `us_national_round_5_2020.png` - 32 regions
- `us_national_round_6_2020.png` - 64 regions
- Continue for later rounds

##### Visual Specifications
- Use consistent color scheme across rounds
- Show state boundaries with districts/regions overlaid
- As states complete their final districts, show them fully divided in subsequent rounds
- Size: 20x12 inches, DPI: 150
- Title: "U.S. Congressional Districts - Round N (2^N regions)"

#### Pipeline Integration
Add as post-processing step after `create_us_rounds_hierarchy.py`:
```python
# CREATE US NATIONAL ROUND PROGRESSION
if output_dir.exists() or args.print_only:
    pipeline_steps.append({
        'name': 'Create national round progression maps',
        'command': f'{sys.executable} scripts/pipeline/create_us_national_rounds_progression.py --year {args.year} --version {args.version} --output-dir {output_dir} --dpi {args.dpi}'.strip(),
        'critical': False
    })
```

#### Dashboard Integration
Add to USA row, Rounds tab:
- Show progressive sequence of national bisection
- Allow users to see national-level recursive division pattern
- Display maps in order: Round 1 → Round 2 → ... → Final

### Benefits
- Visualize national-level recursive bisection strategy
- Understand how equal-population constraint affects regional divisions
- Compare bisection patterns across geographic regions
- Educational tool for understanding METIS recursive algorithm at scale

### Estimated Complexity
**Medium** (2-3 hours)
- Similar to existing national map generation
- Complexity in aggregating round data across states with different completion points
