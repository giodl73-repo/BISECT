#!/usr/bin/env python3
"""
Prepare presidential election data for MMD simulation.

Converts tract-level election data to required format:
geoid, dem_votes, rep_votes, lib_votes, grn_votes, oth_votes
"""

import pandas as pd
from pathlib import Path

def prepare_election_data(year: int = 2020):
    """Prepare election data for MMD simulation."""

    print(f"Preparing {year} presidential election data...")

    # Load raw election data
    input_path = Path(f"data/{year}/elections/2020_president/tracts-2020-RLCR.csv")

    if not input_path.exists():
        raise FileNotFoundError(f"Election data not found: {input_path}")

    df = pd.read_csv(input_path, encoding='utf-8-sig')  # Handle BOM
    print(f"Loaded {len(df):,} tracts")

    # Map columns
    # G20PREDBID = Biden (Democrat)
    # G20PRERTRU = Trump (Republican)
    # G20PRELJOR = Jorgensen (Libertarian)
    # G20PREGHAW = Hawkins (Green)
    # All others = other

    major_candidates = ['G20PREDBID', 'G20PRERTRU', 'G20PRELJOR', 'G20PREGHAW']

    # Create output dataframe
    output = pd.DataFrame({
        'geoid': df['tract_GEOID'].astype(str).str.zfill(11),  # Pad to 11 digits
        'dem_votes': df['G20PREDBID'].fillna(0),
        'rep_votes': df['G20PRERTRU'].fillna(0),
        'lib_votes': df['G20PRELJOR'].fillna(0),
        'grn_votes': df['G20PREGHAW'].fillna(0),
    })

    # Sum all other candidates as "other"
    vote_cols = [col for col in df.columns if col.startswith('G20PRE') and col not in major_candidates]
    output['oth_votes'] = df[vote_cols].fillna(0).sum(axis=1)

    # Filter out tracts with zero votes (may be unpopulated)
    total_votes = output[['dem_votes', 'rep_votes', 'lib_votes', 'grn_votes', 'oth_votes']].sum(axis=1)
    output = output[total_votes > 0].copy()

    print(f"Tracts with votes: {len(output):,}")
    print(f"Total votes: {total_votes.sum():,.0f}")
    print(f"\nVote shares:")
    print(f"  Democrat: {output['dem_votes'].sum() / total_votes.sum() * 100:.2f}%")
    print(f"  Republican: {output['rep_votes'].sum() / total_votes.sum() * 100:.2f}%")
    print(f"  Libertarian: {output['lib_votes'].sum() / total_votes.sum() * 100:.2f}%")
    print(f"  Green: {output['grn_votes'].sum() / total_votes.sum() * 100:.2f}%")
    print(f"  Other: {output['oth_votes'].sum() / total_votes.sum() * 100:.2f}%")

    # Save
    output_path = Path(f"data/{year}/elections/presidential_by_tract.csv")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output.to_csv(output_path, index=False)

    print(f"\nSaved: {output_path}")
    return output


if __name__ == '__main__':
    prepare_election_data(2020)
