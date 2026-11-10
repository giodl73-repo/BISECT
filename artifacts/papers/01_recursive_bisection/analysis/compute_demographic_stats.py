#!/usr/bin/env python3
"""
Aggregate demographic statistics across all congressional districts.
"""

import pandas as pd
from pathlib import Path

def main():
    """Compute national demographic statistics."""

    output_dir = Path('outputs/us_2020_v1/states')

    # Collect all district demographic files
    all_districts = []

    for state_dir in sorted(output_dir.iterdir()):
        if not state_dir.is_dir():
            continue

        demo_file = state_dir / 'demographic_analysis' / 'district_demographics.csv'

        if demo_file.exists():
            df = pd.read_csv(demo_file)
            df['state'] = state_dir.name.replace('_', ' ').title()
            all_districts.append(df)
            print(f"Loaded {len(df)} districts from {df['state'].iloc[0]}")

    # Combine all districts
    df_all = pd.concat(all_districts, ignore_index=True)

    print(f"\n{'='*70}")
    print(f"NATIONAL DEMOGRAPHIC STATISTICS ({len(df_all)} districts)")
    print(f"{'='*70}\n")

    # Overall population
    total_pop = df_all['population'].sum()
    print(f"Total Population: {total_pop:,}")
    print(f"Mean District Population: {df_all['population'].mean():,.0f}")
    print()

    # Race/Ethnicity breakdown
    print("NATIONAL RACIAL/ETHNIC COMPOSITION")
    print("-" * 70)

    race_cols = ['white', 'black', 'asian', 'hispanic', 'other']
    race_totals = {col: df_all[col].sum() for col in race_cols}

    for race, total in race_totals.items():
        pct = 100 * total / total_pop
        print(f"{race.capitalize():15s}: {total:12,} ({pct:5.2f}%)")

    print()

    # Gender breakdown
    print("GENDER COMPOSITION")
    print("-" * 70)
    male_total = df_all['male'].sum()
    female_total = df_all['female'].sum()
    print(f"Male:   {male_total:12,} ({100*male_total/total_pop:.2f}%)")
    print(f"Female: {female_total:12,} ({100*female_total/total_pop:.2f}%)")
    print()

    # District-level statistics
    print("DISTRICT-LEVEL RACIAL COMPOSITION")
    print("-" * 70)

    for col in ['white_pct', 'black_pct', 'asian_pct', 'hispanic_pct']:
        race = col.replace('_pct', '').capitalize()
        mean_pct = df_all[col].mean()
        median_pct = df_all[col].median()
        max_pct = df_all[col].max()

        print(f"{race:10s} - Mean: {mean_pct:5.1f}%  Median: {median_pct:5.1f}%  Max: {max_pct:5.1f}%")

    print()

    # Majority race distribution
    print("MAJORITY RACE BY DISTRICT")
    print("-" * 70)

    majority_counts = df_all['majority_race'].value_counts()
    for race, count in majority_counts.items():
        pct = 100 * count / len(df_all)
        print(f"{race:15s}: {count:3d} districts ({pct:5.1f}%)")

    print()

    # Minority-majority districts
    print("MINORITY-MAJORITY DISTRICTS")
    print("-" * 70)

    minority_maj = df_all[df_all['minority_majority'] == 'Yes']
    print(f"Total minority-majority districts: {len(minority_maj)} ({100*len(minority_maj)/len(df_all):.1f}%)")

    if len(minority_maj) > 0:
        print("\nBreakdown by majority race:")
        for race, count in minority_maj['majority_race'].value_counts().items():
            pct = 100 * count / len(minority_maj)
            print(f"  {race:15s}: {count:2d} districts ({pct:5.1f}%)")

        print(f"\nMean majority race percentage: {minority_maj['majority_race_pct'].mean():.1f}%")
        print(f"Median majority race percentage: {minority_maj['majority_race_pct'].median():.1f}%")

    print()

    # Diversity metrics
    print("DIVERSITY METRICS")
    print("-" * 70)

    # Calculate effective number of races per district (using Simpson diversity)
    def simpson_diversity(row):
        """Calculate Simpson's diversity index (effective number of groups)."""
        proportions = [row[col]/100 for col in ['white_pct', 'black_pct', 'asian_pct', 'hispanic_pct', 'other_pct']]
        simpson = 1 - sum(p**2 for p in proportions)
        # Convert to effective number of groups
        effective_n = 1 / (1 - simpson) if simpson < 1 else float('inf')
        return effective_n

    df_all['diversity_index'] = df_all.apply(simpson_diversity, axis=1)

    print(f"Mean diversity index: {df_all['diversity_index'].mean():.2f}")
    print(f"Median diversity index: {df_all['diversity_index'].median():.2f}")
    print(f"(1 = homogeneous, higher = more diverse)")
    print()

    # Districts by diversity level
    high_diversity = df_all[df_all['diversity_index'] >= 2.5]
    medium_diversity = df_all[(df_all['diversity_index'] >= 1.5) & (df_all['diversity_index'] < 2.5)]
    low_diversity = df_all[df_all['diversity_index'] < 1.5]

    print(f"High diversity (>=2.5):  {len(high_diversity):3d} districts ({100*len(high_diversity)/len(df_all):.1f}%)")
    print(f"Medium diversity (1.5-2.5): {len(medium_diversity):3d} districts ({100*len(medium_diversity)/len(df_all):.1f}%)")
    print(f"Low diversity (<1.5): {len(low_diversity):3d} districts ({100*len(low_diversity)/len(df_all):.1f}%)")
    print()

    # Save aggregated data
    output_file = Path('paper/data/national_demographic_stats.csv')
    output_file.parent.mkdir(parents=True, exist_ok=True)

    # Create summary table
    summary = pd.DataFrame({
        'Metric': ['Total Population', 'Mean District Population',
                   'White %', 'Black %', 'Asian %', 'Hispanic %', 'Other %',
                   'Male %', 'Female %',
                   'White-majority districts', 'Black-majority districts',
                   'Hispanic-majority districts', 'Asian-majority districts',
                   'Minority-majority districts',
                   'Mean diversity index', 'High diversity districts'],
        'Value': [
            f"{total_pop:,}",
            f"{df_all['population'].mean():,.0f}",
            f"{100*race_totals['white']/total_pop:.1f}%",
            f"{100*race_totals['black']/total_pop:.1f}%",
            f"{100*race_totals['asian']/total_pop:.1f}%",
            f"{100*race_totals['hispanic']/total_pop:.1f}%",
            f"{100*race_totals['other']/total_pop:.1f}%",
            f"{100*male_total/total_pop:.1f}%",
            f"{100*female_total/total_pop:.1f}%",
            f"{majority_counts.get('White', 0)} ({100*majority_counts.get('White', 0)/len(df_all):.1f}%)",
            f"{majority_counts.get('Black', 0)} ({100*majority_counts.get('Black', 0)/len(df_all):.1f}%)",
            f"{majority_counts.get('Hispanic', 0)} ({100*majority_counts.get('Hispanic', 0)/len(df_all):.1f}%)",
            f"{majority_counts.get('Asian', 0)} ({100*majority_counts.get('Asian', 0)/len(df_all):.1f}%)",
            f"{len(minority_maj)} ({100*len(minority_maj)/len(df_all):.1f}%)",
            f"{df_all['diversity_index'].mean():.2f}",
            f"{len(high_diversity)} ({100*len(high_diversity)/len(df_all):.1f}%)"
        ]
    })

    summary.to_csv(output_file, index=False)
    print(f"Summary statistics saved to: {output_file}")

    # Save full district-level data
    df_all.to_csv('paper/data/all_districts_demographics.csv', index=False)
    print(f"Full district data saved to: paper/data/all_districts_demographics.csv")

if __name__ == '__main__':
    main()
