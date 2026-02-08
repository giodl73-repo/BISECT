"""
CORRECT Multi-Constraint Implementation
Fixes the bugs identified in Karypis's review
"""

import numpy as np
from typing import List, Tuple


def create_target_weights_CORRECT(
    k: int,
    target_mm: int,
    total_pop: float,
    total_minority: float,
    desired_mm_concentration: float = 0.60
) -> np.ndarray:
    """
    Create correct target weights for multi-constraint partitioning.

    Goal: Achieve desired_mm_concentration (e.g., 60%) minority in target_mm districts

    Args:
        k: Number of districts
        target_mm: Number of target MM districts
        total_pop: Total population
        total_minority: Total minority VAP
        desired_mm_concentration: Desired minority concentration in MM districts (default 0.60)

    Returns:
        tpwgts: Array of shape (k, 2) with [pop_fraction, minority_fraction] per district

    Mathematical Derivation:
    -----------------------
    Given:
    - Each district has equal population: P_i = P_total / k
    - Want MM districts with concentration: M_i / P_i = c_mm
    - Conservation: sum(M_i) = M_total

    For MM districts:
        M_mm = c_mm * P_i = c_mm * (P_total / k)

    For other districts (from conservation):
        N * M_mm + (k-N) * M_other = M_total
        M_other = (M_total - N * M_mm) / (k - N)
                = (M_total - N * c_mm * P_total/k) / (k - N)

        Concentration in other districts:
        c_other = M_other / P_i
                = M_other / (P_total / k)
                = k * M_other / P_total

    For METIS tpwgts (fractions of total):
        frac_mm = M_mm / M_total
                = c_mm * (P_total / k) / M_total
                = c_mm / (k * m_state)
                where m_state = M_total / P_total

        frac_other = M_other / M_total
    """

    # State-wide minority fraction
    m_state = total_minority / total_pop
    pop_per_district = total_pop / k

    # Check feasibility
    # Maximum possible concentration if ALL minority in target_mm districts:
    max_possible = (total_minority / target_mm) / pop_per_district

    if desired_mm_concentration > max_possible:
        print(f"WARNING: Desired {desired_mm_concentration:.1%} concentration is impossible!")
        print(f"  Maximum possible: {max_possible:.1%} (if all minority in {target_mm} districts)")
        print(f"  Using maximum possible instead.")
        desired_mm_concentration = max_possible * 0.95  # Use 95% of max to be safe

    # Calculate minority VAP per district type
    minority_per_mm = desired_mm_concentration * pop_per_district

    # Other districts from conservation
    minority_per_other = (total_minority - target_mm * minority_per_mm) / (k - target_mm)

    if minority_per_other < 0:
        print(f"ERROR: Negative minority for other districts!")
        print(f"  This means desired concentration {desired_mm_concentration:.1%} is too high")
        print(f"  Falling back to equal distribution")
        minority_per_mm = total_minority / k
        minority_per_other = total_minority / k

    # Create tpwgts array
    tpwgts = np.zeros((k, 2))

    # Population targets (equal for all districts)
    tpwgts[:, 0] = 1.0 / k

    # Minority targets (concentrated in first target_mm districts)
    for i in range(target_mm):
        tpwgts[i, 1] = minority_per_mm / total_minority

    for i in range(target_mm, k):
        tpwgts[i, 1] = minority_per_other / total_minority

    # Verify normalization
    assert abs(tpwgts[:, 0].sum() - 1.0) < 1e-6, "Population fractions must sum to 1.0"
    assert abs(tpwgts[:, 1].sum() - 1.0) < 1e-6, "Minority fractions must sum to 1.0"

    # Calculate and report resulting concentrations
    conc_mm = minority_per_mm / pop_per_district
    conc_other = minority_per_other / pop_per_district

    print(f"\nTarget Weights Created:")
    print(f"  MM districts ({target_mm}): {conc_mm:.1%} concentration")
    print(f"  Other districts ({k-target_mm}): {conc_other:.1%} concentration")
    print(f"  tpwgts[MM, minority] = {tpwgts[0, 1]:.4f}")
    print(f"  tpwgts[other, minority] = {tpwgts[target_mm, 1]:.4f}")

    return tpwgts


def run_multi_constraint_CORRECT(
    state_code: str,
    tracts,  # GeoDataFrame
    k: int,
    target_mm: int,
    ubvec: List[float]
):
    """
    CORRECT implementation of multi-constraint partitioning.

    Key fixes:
    1. Actually USE create_target_weights() (Bug #1)
    2. Use correct formula for target weights (Bug #2)
    3. Pass target_weights to METIS (Bug #1)
    """

    # Calculate totals
    total_pop = tracts['total_pop'].sum()
    total_minority = tracts['minority_vap'].sum()

    # Create CORRECT target weights
    target_weights = create_target_weights_CORRECT(
        k=k,
        target_mm=target_mm,
        total_pop=total_pop,
        total_minority=total_minority,
        desired_mm_concentration=0.60
    )

    # Create 2D vertex weights
    vertex_weights_2d = []
    for _, tract in tracts.iterrows():
        vertex_weights_2d.append([
            tract['total_pop'],
            tract['minority_vap']
        ])

    # Run METIS WITH target_weights (FIX BUG #1)
    partition = partition_graph_with_executable(
        adjacency_list,
        vertex_weights_2d,
        nparts=k,
        target_weights=target_weights,  # ← THIS WAS MISSING!
        ubvec=ubvec,
        niter=100
    )

    return partition


# Example usage
if __name__ == "__main__":
    print("=== ALABAMA EXAMPLE ===\n")

    # Alabama parameters
    k = 7
    target_mm = 2
    total_pop = 5_024_279
    total_minority = 1_853_000  # 36.9%

    print(f"State: Alabama")
    print(f"Total pop: {total_pop:,}")
    print(f"Total minority: {total_minority:,} ({100*total_minority/total_pop:.1f}%)")
    print(f"Districts: {k}")
    print(f"Target MM: {target_mm}")

    # Create correct target weights
    tpwgts = create_target_weights_CORRECT(k, target_mm, total_pop, total_minority, 0.60)

    print("\n=== COMPARISON TO BUGGY VERSION ===\n")

    # Buggy version (from Equation 3)
    minority_per_mm_buggy = 0.60 * (total_minority / k)
    minority_per_other_buggy = (total_minority - target_mm * minority_per_mm_buggy) / (k - target_mm)

    pop_per_district = total_pop / k
    conc_mm_buggy = minority_per_mm_buggy / pop_per_district
    conc_other_buggy = minority_per_other_buggy / pop_per_district

    print("BUGGY formula (Equation 3 in paper):")
    print(f"  MM districts: {100*conc_mm_buggy:.1f}% concentration")
    print(f"  Other districts: {100*conc_other_buggy:.1f}% concentration")
    print(f"  Problem: MM < Other (INVERTED!)")

    print("\nCORRECT formula:")
    minority_per_mm_correct = 0.60 * pop_per_district
    minority_per_other_correct = (total_minority - target_mm * minority_per_mm_correct) / (k - target_mm)
    conc_mm_correct = minority_per_mm_correct / pop_per_district
    conc_other_correct = minority_per_other_correct / pop_per_district

    print(f"  MM districts: {100*conc_mm_correct:.1f}% concentration")
    print(f"  Other districts: {100*conc_other_correct:.1f}% concentration")
    print(f"  Success: MM > Other (CORRECT!)")

    print(f"\nError: {100*(0.60 - conc_mm_buggy):.1f} percentage points!")
    print(f"This is a {100*(0.60 - conc_mm_buggy)/0.60:.0f}% error!")
