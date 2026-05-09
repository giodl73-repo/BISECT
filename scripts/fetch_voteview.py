"""
fetch_voteview.py — Download Voteview DW-NOMINATE scores for O.3 analysis.

Voteview.com provides free CSV downloads of congressional DW-NOMINATE
ideal point estimates for all members from 1789 to present.

Usage:
    python scripts/fetch_voteview.py --out data/voteview/nominate_members.csv

Data source:
    Voteview.com — Lewis, Jeffrey B., Keith Poole, Howard Rosenthal,
    Adam Boche, Aaron Rudkin, and Luke Sonnet (2024). Voteview: Congressional
    Roll-Call Votes Database. https://voteview.com/data

Output:
    data/voteview/nominate_members.csv
    Columns: congress, chamber, bioname, nominate_dim1, nominate_dim2,
             state_abbrev, district_code, party_code, born, died
"""

import argparse
import logging
import urllib.request
import io
import pandas as pd
from pathlib import Path

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
log = logging.getLogger(__name__)

# Voteview direct CSV URL — no API key required
VOTEVIEW_URL = "https://voteview.com/static/data/out/members/HSall_members.csv"

# Congress range corresponding to 2001-2024 (107th-118th Congress)
CONGRESS_MIN = 107  # 107th Congress: 2001-2003
CONGRESS_MAX = 118  # 118th Congress: 2023-2025


def fetch_voteview(out_path: Path, congress_min: int = CONGRESS_MIN,
                   congress_max: int = CONGRESS_MAX) -> None:
    """Download and filter Voteview DW-NOMINATE members data."""
    out_path.parent.mkdir(parents=True, exist_ok=True)

    log.info(f"Downloading Voteview members from {VOTEVIEW_URL}...")
    try:
        with urllib.request.urlopen(VOTEVIEW_URL, timeout=60) as resp:
            data = resp.read()
        df = pd.read_csv(io.BytesIO(data), low_memory=False)
        log.info(f"Downloaded {len(df):,} member-congress records")
    except Exception as e:
        log.error(f"Download failed: {e}")
        raise

    # Filter to House members in the relevant congress range
    house = df[
        (df["chamber"] == "House") &
        (df["congress"] >= congress_min) &
        (df["congress"] <= congress_max)
    ].copy()

    # Keep relevant columns
    keep_cols = [
        "congress", "chamber", "icpsr", "state_icpsr", "district_code",
        "state_abbrev", "party_code", "occupancy", "last_means",
        "bioname", "bioguide_id", "born", "died",
        "nominate_dim1", "nominate_dim2",
        "nominate_log_likelihood", "nominate_number_of_votes",
        "nominate_number_of_errors",
    ]
    house = house[[c for c in keep_cols if c in house.columns]]

    # Add ideological extremity measure (|DW-NOMINATE dim1|)
    house["nominate_abs_dim1"] = house["nominate_dim1"].abs()

    # Add party label
    house["party_label"] = house["party_code"].map({
        100: "Democrat", 200: "Republican", 328: "Independent"
    }).fillna("Other")

    # Add congress year range
    house["congress_start_year"] = 1787 + 2 * house["congress"]
    house["congress_end_year"] = house["congress_start_year"] + 2

    log.info(f"Filtered to {len(house):,} House members (congresses {congress_min}-{congress_max})")
    log.info(f"Party breakdown:\n{house['party_label'].value_counts()}")

    house.to_csv(out_path, index=False)
    log.info(f"Wrote {out_path}")

    # Summary statistics
    log.info("\nDW-NOMINATE dim1 summary by party and congress decade:")
    house["decade"] = (house["congress_start_year"] // 10) * 10
    summary = house.groupby(["decade", "party_label"])["nominate_abs_dim1"].agg(["mean", "std", "count"])
    log.info(f"\n{summary.to_string()}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Fetch Voteview DW-NOMINATE scores")
    parser.add_argument("--out", default="data/voteview/nominate_members.csv")
    parser.add_argument("--congress-min", type=int, default=CONGRESS_MIN)
    parser.add_argument("--congress-max", type=int, default=CONGRESS_MAX)
    args = parser.parse_args()
    fetch_voteview(Path(args.out), args.congress_min, args.congress_max)
