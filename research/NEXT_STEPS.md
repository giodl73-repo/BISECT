# Research Portfolio: Immediate Next Steps

## Status Summary
- ✅ **Paper 1** (Edge-Weighted VRA): COMPLETE
- ⚠️ **Paper 2** (N-Way vs Recursive): PARTIALLY COMPLETE - Missing data for 4 states
- 📋 **Papers 3-7**: PLANNED - Comprehensive PLAN.md files created

## Priority: Fill Data Gaps for Paper 2

### Current Data Status
**Alabama (k=7) - COMPLETE ✅**
- Recursive [3,4] predetermined: 43.0% max, 0 MM districts
- Recursive adaptive: 46.1% max, 0 MM districts
- N-way: 47.3% max, 0 MM districts
- Edge-weighted: 50.8% max, 2 MM districts

**Georgia, Mississippi, Louisiana, South Carolina - MISSING ❌**
Need to run:
1. N-way partitioning
2. Recursive bisection (predetermined trees)
3. Adaptive recursive bisection
4. Collect metrics: max minority %, MM count, edge cut, runtime

### Experiments to Run

#### Script to Create: `scripts/pipeline/test_nway_vs_recursive_comprehensive.py`

```python
"""
Comprehensive comparison of n-way vs recursive bisection for VRA compliance.
Tests all 5 states with multiple methods.
"""

import sys
from pathlib import Path
import pandas as pd
import numpy as np
from scipy.sparse import coo_matrix
from collections import defaultdict

project_root = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

from apportionment.partition.metis_wrapper import partition_graph_with_executable
from apportionment.data.adjacency import load_adjacency_matrix

def load_state_data(state_code, year=2020, version='v1'):
    """Load census tracts, demographics, adjacency for a state"""
    from scripts.utils.path_utils import get_tract_file, get_demographics_file, get_adjacency_file

    tracts = pd.read_csv(get_tract_file(state_code, year, version))
    demographics = pd.read_csv(get_demographics_file(state_code, year, version))
    adjacency = load_adjacency_matrix(get_adjacency_file(state_code, year, version))

    tracts_with_demo = tracts.merge(demographics, on='GEOID', how='inner')
    return tracts_with_demo, adjacency

def create_minority_edge_weights(adj_matrix, tracts_with_demo, minority_threshold=0.40, weight_factor=5):
    """Create edge weights that penalize cuts between minority tracts"""
    is_minority = tracts_with_demo['pct_minority'] >= minority_threshold

    edge_weights = {}
    adj_coo = adj_matrix.tocoo()

    for i, j in zip(adj_coo.row, adj_coo.col):
        if i < j:
            if is_minority.iloc[i] and is_minority.iloc[j]:
                edge_weights[(i, j)] = weight_factor
            else:
                edge_weights[(i, j)] = 1.0

    return edge_weights

def run_nway_partitioning(tracts_with_demo, adjacency, num_districts, weight_factor=5, threshold=0.40):
    """N-way partitioning with edge-weighting"""
    vertex_weights = tracts_with_demo['total_pop'].values

    edge_weights = create_minority_edge_weights(
        adjacency, tracts_with_demo, threshold, weight_factor
    )

    adjacency_list = {}
    adj_coo = adjacency.tocoo()
    for i, j in zip(adj_coo.row, adj_coo.col):
        if i not in adjacency_list:
            adjacency_list[i] = []
        if j not in adjacency_list:
            adjacency_list[j] = []
        adjacency_list[i].append(j)
        adjacency_list[j].append(i)

    partition = partition_graph_with_executable(
        adjacency_list,
        vertex_weights,
        nparts=num_districts,
        ufactor=1.005,
        edge_weights=edge_weights,
        niter=100,
        debug=False
    )

    return partition

def run_recursive_bisection_predetermined(tracts_with_demo, adjacency, num_districts,
                                         tree_structure, weight_factor=5, threshold=0.40):
    """
    Recursive bisection with predetermined tree structure.
    tree_structure: e.g., [3, 4] for splitting into 3 and 4 districts
    """
    # This is a simplified version - full implementation would recursively bisect
    # For now, this is a placeholder that would call recursive bisection logic

    # TODO: Implement full recursive bisection with tree structure
    # For now, return None to indicate this needs implementation
    return None

def run_recursive_bisection_adaptive(tracts_with_demo, adjacency, num_districts,
                                    weight_factor=5, threshold=0.40):
    """
    Adaptive recursive bisection - chooses tree structure based on data.
    At each split, tries both orderings and chooses best.
    """
    # TODO: Implement adaptive tree selection
    # For now, return None to indicate this needs implementation
    return None

def analyze_partition(partition, tracts_with_demo, adjacency, target_mm_count):
    """Compute metrics for a partition"""
    # District-level minority percentages
    district_minorities = defaultdict(lambda: {'pop': 0, 'minority_vap': 0})

    for idx, district in enumerate(partition):
        row = tracts_with_demo.iloc[idx]
        district_minorities[district]['pop'] += row['total_pop']
        district_minorities[district]['minority_vap'] += row['minority_vap']

    minority_pcts = []
    for d in district_minorities.values():
        pct = d['minority_vap'] / d['pop'] if d['pop'] > 0 else 0
        minority_pcts.append(pct)

    max_minority_pct = max(minority_pcts)
    mm_count = sum(1 for pct in minority_pcts if pct >= 0.50)

    # Edge cut
    edge_cut = 0
    adj_coo = adjacency.tocoo()
    for i, j in zip(adj_coo.row, adj_coo.col):
        if i < j and partition[i] != partition[j]:
            edge_cut += 1

    success = (mm_count >= target_mm_count)

    return {
        'max_minority_pct': max_minority_pct,
        'mm_count': mm_count,
        'edge_cut': edge_cut,
        'success': success
    }

def main():
    # Test configurations
    states = {
        'mississippi': {'code': 28, 'k': 4, 'target_mm': 2, 'minority_pct': 0.461},
        'georgia': {'code': 13, 'k': 14, 'target_mm': 5, 'minority_pct': 0.424},
        'louisiana': {'code': 22, 'k': 6, 'target_mm': 2, 'minority_pct': 0.416},
        'alabama': {'code': 1, 'k': 7, 'target_mm': 2, 'minority_pct': 0.369},
        'south_carolina': {'code': 45, 'k': 7, 'target_mm': 3, 'minority_pct': 0.351}
    }

    results = []

    for state_name, config in states.items():
        print(f"\n{'='*60}")
        print(f"Testing {state_name.upper()}")
        print(f"{'='*60}")

        # Load data
        print(f"Loading data for {state_name}...")
        tracts_with_demo, adjacency = load_state_data(config['code'])

        # N-way partitioning
        print(f"Running n-way partitioning (k={config['k']})...")
        partition_nway = run_nway_partitioning(
            tracts_with_demo, adjacency, config['k']
        )

        metrics_nway = analyze_partition(
            partition_nway, tracts_with_demo, adjacency, config['target_mm']
        )

        results.append({
            'state': state_name,
            'method': 'nway',
            'k': config['k'],
            'target_mm': config['target_mm'],
            'state_minority_pct': config['minority_pct'],
            **metrics_nway
        })

        print(f"  N-way: {metrics_nway['mm_count']}/{config['target_mm']} MM districts, "
              f"max {metrics_nway['max_minority_pct']:.1%}, edge cut {metrics_nway['edge_cut']}")

        # Recursive bisection - predetermined trees
        # For k=7, test [3,4] as example
        # TODO: Test multiple tree structures
        print(f"Running recursive bisection (predetermined tree)...")
        print(f"  [PLACEHOLDER - Needs implementation]")

        # Recursive bisection - adaptive
        print(f"Running adaptive recursive bisection...")
        print(f"  [PLACEHOLDER - Needs implementation]")

    # Save results
    results_df = pd.DataFrame(results)
    output_path = project_root / 'research' / 'gerry-nway-vs-recursive' / 'results' / 'nway_vs_recursive_comparison.csv'
    output_path.parent.mkdir(parents=True, exist_ok=True)
    results_df.to_csv(output_path, index=False)

    print(f"\n{'='*60}")
    print(f"Results saved to {output_path}")
    print(f"{'='*60}")

    # Summary table
    print("\nSUMMARY:")
    print(results_df.to_string(index=False))

if __name__ == '__main__':
    main()
```

### Implementation Steps

**Step 1: Complete N-Way for 4 States (Week 1, Days 1-2)**
- Run `test_edge_weighting_comprehensive.py` for Georgia, Mississippi, Louisiana, South Carolina
- Extract n-way results (already have from Paper 1, just need to compile)

**Step 2: Implement Recursive Bisection (Week 1, Days 3-4)**
- Create recursive bisection wrapper that calls METIS with tree structure
- Test multiple predetermined trees per state
- Implement adaptive tree selection (try both orderings at each split)

**Step 3: Collect All Metrics (Week 1, Day 5)**
- Run comprehensive comparison script
- Generate results CSV with all methods × all states
- Compute statistical significance tests

**Step 4: Update Paper 2 (Week 2)**
- Fill in results tables (currently have placeholders like "?" and "--")
- Update discussion with cross-state patterns
- Compile LaTeX and check formatting

### Expected Results Based on Alabama

| State | State Minority % | N-Way | Recursive | Adaptive | Gap (N-Way - Recursive) |
|-------|-----------------|-------|-----------|----------|------------------------|
| Mississippi | 46.1% | ~55% | ~52% | ~54% | ~3 pts |
| Georgia | 42.4% | ~51% | ~47% | ~49% | ~4 pts |
| Louisiana | 41.6% | ~50% | ~46% | ~48% | ~4 pts |
| Alabama | 36.9% | 47.3% | 43.0% | 46.1% | 4.3 pts |
| South Carolina | 35.1% | ~45% | ~41% | ~43% | ~4 pts |

**Pattern:** N-way advantage likely ~3-5 points across all states, with adaptive splitting the difference.

## After Filling Paper 2 Data Gaps

### Option A: Continue to Papers 3-4 (Recommended)
**Papers 3-4 build directly on Papers 1-2:**
- Paper 3 (Adaptive): Uses recursive baseline from Paper 2
- Paper 4 (Multi vs Edge): Compares multi-constraint to edge-weighted from Paper 1

**Timeline:** 2 weeks for both papers (can run experiments in parallel)

### Option B: Jump to Paper 5 (Threshold Analysis)
**If interested in policy implications:**
- Paper 5 requires Papers 1 + 4 complete (need multi-constraint results)
- Would need to run multi-constraint experiments first
- Then can identify feasibility threshold

### Option C: Focus on Paper 6 (Compactness Tradeoff)
**If interested in debunking VRA-compactness myth:**
- Requires baseline compactness measurements
- Already have edge-weighted compactness from Paper 1
- Need to add Polsby-Popper, Reock metrics

## Quick Reference

### Key Files
- **Master index:** `research/README.md`
- **Paper 1 results:** `research/gerry-vra-compliance/results/edge_weighting_ablation_study.csv`
- **Paper 2 structure:** `research/gerry-nway-vs-recursive/sections/*.tex`
- **Plans for Papers 3-7:** `research/gerry-{adaptive,multi-vs-edge,threshold,compactness,temporal}/PLAN.md`

### Commands to Run

**Start Paper 2 data collection:**
```bash
# Run n-way for all states (may already have from Paper 1)
python scripts/pipeline/test_edge_weighting_comprehensive.py

# Run recursive bisection comparison (TO CREATE)
python scripts/pipeline/test_nway_vs_recursive_comprehensive.py
```

**Compile Paper 2:**
```bash
cd research/gerry-nway-vs-recursive
pdflatex main.tex
bibtex main
pdflatex main.tex
pdflatex main.tex
```

## Timeline Estimate

- **Week 1:** Fill Paper 2 data gaps (n-way + recursive for 4 states)
- **Weeks 2-3:** Complete Papers 3-4 (adaptive, multi vs edge)
- **Weeks 4-6:** Complete Papers 5-6 (threshold, compactness)
- **Weeks 7-10:** Complete Paper 7 (temporal stability with 2010 data)

**Total:** 10 weeks to complete all 7 papers

## Next Command

To start filling data gaps, create the comprehensive comparison script:

```bash
# Create new experiment script
touch scripts/pipeline/test_nway_vs_recursive_comprehensive.py

# Or directly run existing edge-weighting script to extract n-way results
python scripts/pipeline/test_edge_weighting_comprehensive.py
```

---

**Status:** Ready to begin data collection for Paper 2. All planning complete. ✅
