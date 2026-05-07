"""
Demonstrate party-vote weighting concept without requiring actual tract files.
"""

import numpy as np
import pandas as pd

# Simulate Ohio with 3000 tracts
np.random.seed(42)
n_tracts = 3000

print("="*60)
print("Party-Vote Weighted Districts - Concept Demonstration")
print("="*60)
print(f"\nSimulating Ohio with {n_tracts} census tracts")

# Simulate tract data with geographic clustering
# Urban tracts (first 1000): heavily Democratic
# Suburban tracts (middle 1000): mixed
# Rural tracts (last 1000): heavily Republican

urban_dem_pct = np.random.uniform(0.60, 0.80, 1000)
suburban_dem_pct = np.random.uniform(0.40, 0.60, 1000)
rural_dem_pct = np.random.uniform(0.20, 0.40, 1000)

dem_pct = np.concatenate([urban_dem_pct, suburban_dem_pct, rural_dem_pct])
rep_pct = 1.0 - dem_pct

# Total votes per tract (varying population density)
urban_votes = np.random.uniform(3000, 5000, 1000)
suburban_votes = np.random.uniform(1500, 3000, 1000)
rural_votes = np.random.uniform(500, 1500, 1000)

total_votes = np.concatenate([urban_votes, suburban_votes, rural_votes])

# Calculate party votes
dem_votes = (total_votes * dem_pct).astype(int)
rep_votes = (total_votes * rep_pct).astype(int)

tract_df = pd.DataFrame({
    'tract_id': range(n_tracts),
    'region': ['Urban']*1000 + ['Suburban']*1000 + ['Rural']*1000,
    'total_votes': total_votes.astype(int),
    'dem_votes': dem_votes,
    'rep_votes': rep_votes
})

# Summary stats
print(f"\nStatewide totals:")
total_dem = tract_df['dem_votes'].sum()
total_rep = tract_df['rep_votes'].sum()
total = total_dem + total_rep

print(f"  Democratic: {total_dem:,} ({100*total_dem/total:.1f}%)")
print(f"  Republican: {total_rep:,} ({100*total_rep/total:.1f}%)")
print(f"  Total votes: {total:,}")

# D'Hondt allocation for 15 seats
dem_pct_state = total_dem / total
rep_pct_state = total_rep / total

# Calculate seats (simplified D'Hondt)
dem_seats = round(15 * dem_pct_state)
rep_seats = 15 - dem_seats

print(f"\nD'Hondt allocation (15 seats):")
print(f"  Democratic: {dem_seats} seats ({100*dem_seats/15:.1f}%)")
print(f"  Republican: {rep_seats} seats ({100*rep_seats/15:.1f}%)")

# Now show the difference between total-population and party-vote weighting

print(f"\n{'='*60}")
print("COMPARISON: Total Population vs Party-Vote Weighting")
print('='*60)

# Method 1: Total population weighting (traditional)
print(f"\nMethod 1: TOTAL VOTES weighting (traditional)")
print(f"Each district = {total/15:,.0f} total votes")

ideal_votes_per_district = total / 15

# Simulate districts (simplified - just show imbalance)
# Hypothetical: 7 Democratic districts
dem_districts_traditional = []
for i in range(dem_seats):
    # Each district has equal TOTAL votes
    district_total_votes = ideal_votes_per_district
    # But Democratic votes vary by geography
    if i < 3:  # Urban districts
        dem_votes_in_district = district_total_votes * 0.70  # 70% Dem
    else:  # Some rural districts
        dem_votes_in_district = district_total_votes * 0.40  # 40% Dem

    dem_districts_traditional.append(dem_votes_in_district)

print(f"\nDemocratic districts (total-vote balanced):")
for i, dv in enumerate(dem_districts_traditional, 1):
    pct = 100 * dv / ideal_votes_per_district
    print(f"  District {i}: {dv:>7,.0f} Dem voters ({pct:4.1f}% of district)")

print(f"\nWARNING PROBLEM: Urban districts have {dem_districts_traditional[0]/dem_districts_traditional[-1]:.1f}x more Democratic voters!")
print(f"  -> Unequal representation within Democratic caucus")

# Method 2: Party-vote weighting (our proposal)
print(f"\n{'='*60}")
print(f"Method 2: PARTY-VOTE weighting (our proposal)")
print(f"Each Democratic district = {total_dem/dem_seats:,.0f} Democratic votes")

ideal_dem_votes_per_district = total_dem / dem_seats

dem_districts_party_weighted = [ideal_dem_votes_per_district] * dem_seats

print(f"\nDemocratic districts (Dem-vote balanced):")
for i, dv in enumerate(dem_districts_party_weighted, 1):
    print(f"  District {i}: {dv:>7,.0f} Dem voters (equal representation)")

print(f"\n[OK] SOLUTION: All Democratic districts have equal Democratic voters!")
print(f"  -> Fair representation within Democratic caucus")

# Show how geographic distribution would differ
print(f"\n{'='*60}")
print("Geographic implications:")
print('='*60)

print(f"\nTraditional method (total-vote balanced):")
print(f"  - Democratic districts forced into rural areas")
print(f"  - Many Democratic voters in districts with few co-partisans")
print(f"  - Unequal representation per Democratic voter")

print(f"\nParty-vote weighted method (Dem-vote balanced):")
print(f"  - Democratic districts concentrate where Democrats are")
print(f"  - Each Democratic voter has equal representation weight")
print(f"  - Republican districts separately concentrate where Republicans are")
print(f"  - Districts overlap geographically!")

print(f"\n{'='*60}")
print("Key insight: Party-vote weighting ensures proportional")
print("representation WITHIN each party's delegation, not just")
print("at the statewide level.")
print('='*60)
