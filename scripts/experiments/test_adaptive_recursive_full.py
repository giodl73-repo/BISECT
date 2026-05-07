"""
Complete adaptive recursive bisection for VRA compliance.

Makes data-driven split decisions at EVERY level of recursion by trying
both options and picking whichever achieves better minority concentration.
"""

import argparse
from pathlib import Path
import sys

# Add project root and src to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

import numpy as np
import geopandas as gpd
from typing import List, Tuple
from apportionment.partition import vra_utils
from apportionment.partition.metis_executable import partition_graph_with_executable
from apportionment.data.adjacency import build_adjacency_graph
from scripts.utils import get_state_config, get_tract_file


class AdaptiveRecursiveBisection:
    """Adaptive recursive bisection with data-driven split selection."""

    def __init__(self, tracts_gdf, vertex_weights, adjacency_list,
                 num_districts, target_mm_districts, state_minority_pct,
                 mm_target_pct=0.60, ubvec=None, debug=True):
        self.tracts_gdf = tracts_gdf
        self.vertex_weights = vertex_weights
        self.adjacency_list = adjacency_list
        self.num_districts = num_districts
        self.target_mm_districts = target_mm_districts
        self.state_minority_pct = state_minority_pct
        self.mm_target_pct = mm_target_pct
        self.ubvec = ubvec
        self.debug = debug

        # Track district assignments
        self.assignments = np.full(len(tracts_gdf), -1, dtype=int)
        self.next_district_id = 0

        # Track decisions made
        self.split_decisions = []

    def partition(self):
        """Run adaptive recursive bisection."""
        print("\n" + "="*70)
        print("Starting Adaptive Recursive Bisection")
        print("="*70)
        print(f"Total districts: {self.num_districts}")
        print(f"Target MM districts: {self.target_mm_districts}")
        print(f"State minority: {self.state_minority_pct*100:.1f}%")
        print("="*70 + "\n")

        # Start recursion with all tracts
        tract_indices = np.arange(len(self.tracts_gdf))
        self._adaptive_split(
            tract_indices=tract_indices,
            num_districts=self.num_districts,
            target_mm=self.target_mm_districts,
            depth=0,
            path=""
        )

        return self.assignments

    def _adaptive_split(self, tract_indices, num_districts, target_mm, depth, path):
        """Recursively split with adaptive decision making."""
        indent = "  " * depth

        # Base case: single district
        if num_districts == 1:
            district_id = self.next_district_id
            self.next_district_id += 1
            self.assignments[tract_indices] = district_id

            if self.debug:
                tracts_subset = self.tracts_gdf.iloc[tract_indices]
                total_pop = tracts_subset['total_pop'].sum()
                minority_pop = (tracts_subset['total_pop'] * tracts_subset['pct_minority']).sum()
                pct = minority_pop / total_pop if total_pop > 0 else 0
                mm_mark = "[MM]" if pct >= 0.50 else "    "
                print(f"{indent}{mm_mark} District {district_id}: {len(tract_indices)} tracts, "
                      f"{pct*100:.1f}% minority")

            return

        if self.debug:
            print(f"\n{indent}{'='*60}")
            print(f"{indent}Level {depth}: Splitting {num_districts} districts")
            print(f"{indent}  Tracts: {len(tract_indices)}, Target MM: {target_mm}")
            print(f"{indent}{'='*60}")

        # Determine split options
        if num_districts == 2:
            options = [(1, 1)]
        else:
            left = num_districts // 2
            right = num_districts - left
            if left == right:
                options = [(left, right)]
            else:
                options = [(left, right), (right, left)]

        # Extract subgraph for these tracts
        sub_adjacency = self._extract_subgraph(tract_indices)
        sub_vertex_weights = self.vertex_weights[tract_indices]

        # Try each split option
        best_split = None
        best_concentration = -1
        best_assignments = None
        best_left_mm = None
        best_right_mm = None

        for left_count, right_count in options:
            if self.debug:
                print(f"\n{indent}  Testing [{left_count}, {right_count}]...")

            # Calculate target MM distribution
            # Strategy: put as many MM as possible in left (standard clustering)
            left_mm = min(left_count, target_mm)
            right_mm = target_mm - left_mm

            # Calculate target weights
            target_weights = self._calculate_targets(
                left_count, right_count, left_mm, right_mm, num_districts
            )

            # Run METIS
            assignments = partition_graph_with_executable(
                adjacency=sub_adjacency,
                vertex_weights=sub_vertex_weights,
                nparts=2,
                target_weights=target_weights,
                ufactor=1.005,
                ubvec=self.ubvec,
                niter=100,
                debug=False
            )

            # Measure concentration achieved
            tracts_subset = self.tracts_gdf.iloc[tract_indices].copy()
            tracts_subset['temp_part'] = assignments

            part0 = tracts_subset[tracts_subset['temp_part'] == 0]
            part1 = tracts_subset[tracts_subset['temp_part'] == 1]

            part0_total = part0['total_pop'].sum()
            part0_minority = (part0['total_pop'] * part0['pct_minority']).sum()
            part1_total = part1['total_pop'].sum()
            part1_minority = (part1['total_pop'] * part1['pct_minority']).sum()

            part0_pct = part0_minority / part0_total if part0_total > 0 else 0
            part1_pct = part1_minority / part1_total if part1_total > 0 else 0

            # Score: concentration in the part with MM districts
            # If both have MM, take the average; if one has MM, take that one
            if left_mm > 0 and right_mm > 0:
                concentration = (part0_pct + part1_pct) / 2
            elif left_mm > 0:
                concentration = part0_pct
            else:
                concentration = part1_pct

            if self.debug:
                print(f"{indent}    L: {part0_pct*100:.1f}% ({left_mm} MM), "
                      f"R: {part1_pct*100:.1f}% ({right_mm} MM)")
                print(f"{indent}    Score: {concentration*100:.1f}%")

            if concentration > best_concentration:
                best_concentration = concentration
                best_split = (left_count, right_count)
                best_assignments = assignments
                best_left_mm = left_mm
                best_right_mm = right_mm

        # Record decision
        decision = {
            'depth': depth,
            'path': path,
            'num_districts': num_districts,
            'split': best_split,
            'concentration': best_concentration,
            'left_mm': best_left_mm,
            'right_mm': best_right_mm
        }
        self.split_decisions.append(decision)

        if self.debug:
            print(f"\n{indent}>>> CHOSE [{best_split[0]}, {best_split[1]}] "
                  f"with {best_concentration*100:.1f}% concentration")

        # Split tract indices by assignment
        left_indices = tract_indices[best_assignments == 0]
        right_indices = tract_indices[best_assignments == 1]

        # Recurse on each part
        if self.debug:
            print(f"{indent}Recursing left ({best_split[0]} districts)...")
        self._adaptive_split(left_indices, best_split[0], best_left_mm, depth + 1, path + "0")

        if self.debug:
            print(f"{indent}Recursing right ({best_split[1]} districts)...")
        self._adaptive_split(right_indices, best_split[1], best_right_mm, depth + 1, path + "1")

    def _extract_subgraph(self, tract_indices):
        """Extract adjacency subgraph for subset of tracts."""
        # Create mapping from original to subgraph indices
        old_to_new = {old_idx: new_idx for new_idx, old_idx in enumerate(tract_indices)}

        # Build subgraph adjacency
        sub_adjacency = []
        for old_idx in tract_indices:
            neighbors = self.adjacency_list[old_idx]
            # Only include neighbors that are in the subset
            sub_neighbors = [old_to_new[n] for n in neighbors if n in old_to_new]
            sub_adjacency.append(sub_neighbors)

        return sub_adjacency

    def _calculate_targets(self, left_count, right_count, left_mm, right_mm, total_districts):
        """Calculate target weights for a split."""
        pop_per_district = 1.0 / total_districts

        # Calculate minority needs
        left_mm_minority = left_mm * pop_per_district * self.mm_target_pct
        right_mm_minority = right_mm * pop_per_district * self.mm_target_pct

        # Remaining minority for non-MM
        total_mm_minority = (left_mm + right_mm) * pop_per_district * self.mm_target_pct
        remaining_minority = self.state_minority_pct - total_mm_minority

        non_mm_total = total_districts - self.target_mm_districts
        left_non_mm = left_count - left_mm
        right_non_mm = right_count - right_mm

        left_non_mm_minority = (left_non_mm * remaining_minority / non_mm_total) if non_mm_total > 0 else 0
        right_non_mm_minority = (right_non_mm * remaining_minority / non_mm_total) if non_mm_total > 0 else 0

        # Total for each side
        left_minority = left_mm_minority + left_non_mm_minority
        right_minority = right_mm_minority + right_non_mm_minority

        # As fractions
        left_minority_frac = left_minority / self.state_minority_pct if self.state_minority_pct > 0 else 0.5
        right_minority_frac = right_minority / self.state_minority_pct if self.state_minority_pct > 0 else 0.5

        left_pop_frac = left_count / total_districts
        right_pop_frac = right_count / total_districts

        # Normalize to ensure they sum to 1.0 (handle floating point errors)
        pop_sum = left_pop_frac + right_pop_frac
        minority_sum = left_minority_frac + right_minority_frac

        if pop_sum > 0:
            left_pop_frac /= pop_sum
            right_pop_frac /= pop_sum

        if minority_sum > 0:
            left_minority_frac /= minority_sum
            right_minority_frac /= minority_sum

        return [
            [left_pop_frac, left_minority_frac],
            [right_pop_frac, right_minority_frac]
        ]


def main():
    parser = argparse.ArgumentParser(description='Full adaptive recursive bisection for VRA')
    parser.add_argument('--state', type=str, required=True, help='State code (e.g., AL)')
    parser.add_argument('--num-districts', type=int, required=True, help='Number of districts')
    parser.add_argument('--target-mm-districts', type=int, required=True, help='Target MM districts')
    parser.add_argument('--year', type=int, default=2020, help='Census year')
    parser.add_argument('--mm-target-pct', type=float, default=0.60, help='Target minority % for MM districts')
    parser.add_argument('--ubvec', type=float, default=None, help='Minority constraint imbalance tolerance (e.g., 1000 for VRA)')
    parser.add_argument('--quiet', action='store_true', help='Reduce output verbosity')

    args = parser.parse_args()

    # Map state code to name
    state_map = {'AL': 'alabama', 'GA': 'georgia', 'LA': 'louisiana', 'MS': 'mississippi', 'SC': 'south_carolina'}
    state_name = state_map.get(args.state)
    if not state_name:
        print(f"ERROR: Unknown state code: {args.state}")
        return 1

    # Get state minority percentage
    state_minority_pcts = {'alabama': 0.369, 'georgia': 0.424, 'louisiana': 0.416, 'mississippi': 0.461, 'south_carolina': 0.351}
    state_minority_pct = state_minority_pcts[state_name]

    print(f"\n{'='*70}")
    print(f"Full Adaptive Recursive Bisection - {state_name.title()}")
    print(f"{'='*70}")
    print(f"  State: {args.state} ({state_name.title()})")
    print(f"  Districts: {args.num_districts}")
    print(f"  Target MM districts: {args.target_mm_districts}")
    print(f"  State minority %: {state_minority_pct*100:.1f}%")
    print(f"  MM target %: {args.mm_target_pct*100:.1f}%")
    print(f"  Year: {args.year}")
    print()

    # Get tract file
    tracts_file = str(get_tract_file(args.state, str(args.year), 'v1'))

    # Load tract data
    print("Loading tract data...")
    tracts_gdf = gpd.read_parquet(tracts_file)
    print(f"  Loaded {len(tracts_gdf)} tracts")

    # Load demographics
    print("Loading demographics...")
    demographics = vra_utils.load_tract_demographics(state_name, args.year)

    # Create VRA vertex weights
    print("Creating VRA vertex weights...")
    vertex_weights_vra, tracts_with_demo = vra_utils.create_vra_vertex_weights(
        tracts_gdf, demographics
    )
    print(f"  Created weights for {len(tracts_with_demo)} tracts")

    # Build adjacency
    print("Building adjacency...")
    adjacency_list, _, _, _, _ = build_adjacency_graph(tracts_with_demo)
    print(f"  Built adjacency graph with {len(adjacency_list)} nodes")

    # Build ubvec if specified
    if args.ubvec:
        ubvec = [1.005, args.ubvec]
        print(f"Using ubvec: population={ubvec[0]}, minority={ubvec[1]}")
    else:
        ubvec = None
        print("Using tpwgts only (no ubvec)")

    # Run adaptive recursive bisection
    partitioner = AdaptiveRecursiveBisection(
        tracts_gdf=tracts_with_demo,
        vertex_weights=vertex_weights_vra,
        adjacency_list=adjacency_list,
        num_districts=args.num_districts,
        target_mm_districts=args.target_mm_districts,
        state_minority_pct=state_minority_pct,
        mm_target_pct=args.mm_target_pct,
        ubvec=ubvec,
        debug=not args.quiet
    )

    assignments = partitioner.partition()

    # Analyze results
    print("\n" + "="*70)
    print("FINAL RESULTS")
    print("="*70)

    tracts_with_demo['district'] = assignments
    vra_analysis = vra_utils.analyze_mm_districts(
        tracts_with_demo, assignments, mm_threshold=0.50
    )

    mm_count = vra_analysis['mm_count']
    print(f"MM Districts Created: {mm_count} / {args.target_mm_districts} (target)")
    print()

    # Show all districts sorted by minority %
    districts = sorted(vra_analysis['districts'], key=lambda d: d['pct_minority'], reverse=True)
    for dist in districts:
        mm_mark = "[MM]" if dist['is_mm'] else "     "
        print(f"{mm_mark} District {dist['district']}: {dist['pct_minority']*100:.1f}% minority")

    # Show decision tree
    print(f"\n{'='*70}")
    print("DECISION TREE")
    print(f"{'='*70}")
    for decision in partitioner.split_decisions:
        indent = "  " * decision['depth']
        print(f"{indent}{decision['num_districts']} -> [{decision['split'][0]}, {decision['split'][1]}] "
              f"(L:{decision['left_mm']} MM, R:{decision['right_mm']} MM, "
              f"score:{decision['concentration']*100:.1f}%)")

    print(f"\n{'='*70}")
    if mm_count >= args.target_mm_districts:
        print(f"SUCCESS: Achieved {args.target_mm_districts} MM districts!")
    elif mm_count > 0:
        print(f"PARTIAL: Achieved {mm_count} MM district(s), target was {args.target_mm_districts}")
    else:
        print(f"NO MM DISTRICTS: Adaptive approach could not create MM districts")
    print(f"{'='*70}\n")

    return 0


if __name__ == '__main__':
    sys.exit(main())
