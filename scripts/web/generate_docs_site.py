#!/usr/bin/env python3
"""
Generate educational docs site assets.

This script prepares assets for the educational website by:
1. Copying key figures from artifacts/ and outputs/
2. Organizing them by chapter
3. Generating a JSON manifest for dynamic loading
4. Converting Mermaid diagrams to SVG (future)

Usage:
    python scripts/web/generate_docs_site.py [--dry-run]
"""

import argparse
import json
import shutil
from pathlib import Path
from typing import Dict, List


# Paths
PROJECT_ROOT = Path(__file__).parent.parent.parent
DOCS_PUBLIC = PROJECT_ROOT / "web" / "docs" / "public"
ARTIFACTS = PROJECT_ROOT / "artifacts"
OUTPUTS = PROJECT_ROOT / "outputs"
RESEARCH = PROJECT_ROOT / "research"

# Asset mapping: (source_path, destination_path, chapter)
ASSET_MAP = [
    # Chapter 1: Tracts to Graphs - Alabama
    (
        OUTPUTS / "V1" / "2020" / "states" / "alabama" / "maps" / "all_districts.png",
        DOCS_PUBLIC / "figures" / "chapter1" / "alabama_tracts.png",
        1,
    ),
    (
        ARTIFACTS / "papers" / "03_combined_recursive_bisection" / "figures" / "adjacency_process.png",
        DOCS_PUBLIC / "figures" / "chapter1" / "adjacency_process.png",
        1,
    ),

    # Chapter 2: Splitting - Alabama Round 1
    (
        OUTPUTS / "V1" / "2020" / "states" / "alabama" / "maps" / "rounds" / "round_01.png",
        DOCS_PUBLIC / "figures" / "chapter2" / "alabama_round_1_2_regions.png",
        2,
    ),

    # Chapter 3: Recursion - Alabama Rounds 1-3
    (
        OUTPUTS / "V1" / "2020" / "states" / "alabama" / "maps" / "rounds" / "round_01.png",
        DOCS_PUBLIC / "figures" / "chapter3" / "alabama_round_1_2_regions.png",
        3,
    ),
    (
        OUTPUTS / "V1" / "2020" / "states" / "alabama" / "maps" / "rounds" / "round_02.png",
        DOCS_PUBLIC / "figures" / "chapter3" / "alabama_round_2_4_regions.png",
        3,
    ),
    (
        OUTPUTS / "V1" / "2020" / "states" / "alabama" / "maps" / "rounds" / "round_03.png",
        DOCS_PUBLIC / "figures" / "chapter3" / "alabama_round_3_7_districts.png",
        3,
    ),
    # Chapter 3: Alabama vs Colorado Comparison
    (
        OUTPUTS / "V1" / "2020" / "states" / "alabama" / "maps" / "all_districts.png",
        DOCS_PUBLIC / "figures" / "chapter3" / "alabama_final.png",
        3,
    ),
    (
        OUTPUTS / "V1" / "2020" / "states" / "colorado" / "maps" / "all_districts.png",
        DOCS_PUBLIC / "figures" / "chapter3" / "colorado_final.png",
        3,
    ),

    # Chapter 4: Compactness
    (
        ARTIFACTS / "papers" / "03_combined_recursive_bisection" / "figures" / "national_comparison_bar.png",
        DOCS_PUBLIC / "figures" / "chapter4" / "national_comparison_bar.png",
        4,
    ),
    (
        ARTIFACTS / "papers" / "03_combined_recursive_bisection" / "figures" / "state_scatter.png",
        DOCS_PUBLIC / "figures" / "chapter4" / "state_scatter.png",
        4,
    ),

    # Chapter 5: VRA
    (
        RESEARCH / "03+vra-compliance" / "figures" / "approach_comparison.png",
        DOCS_PUBLIC / "figures" / "chapter5" / "figure1_success_rates.png",
        5,
    ),
    (
        RESEARCH / "04+threshold-analysis" / "results" / "figure1_50state_threshold.png",
        DOCS_PUBLIC / "figures" / "chapter5" / "figure1_50state_threshold.png",
        5,
    ),

    # Chapter 6: Edge-Factor
    (
        RESEARCH / "03+vra-compliance" / "figures" / "vra_compactness_tradeoff.png",
        DOCS_PUBLIC / "figures" / "chapter6" / "figure2_compactness_tradeoff.png",
        6,
    ),
]

# Research papers to copy
PAPERS = [
    ("01_comparison_plans", "papers/01_comparison_plans.pdf"),
    ("02_edge_weighted_bisection", "papers/02_edge_weighted_bisection.pdf"),
    ("03_combined_recursive_bisection", "papers/03_combined_recursive_bisection.pdf"),
    # Add more as papers are finalized
]


def copy_asset(source: Path, dest: Path, dry_run: bool = False) -> bool:
    """Copy a single asset file."""
    if not source.exists():
        print(f"[SKIP] Source not found: {source}")
        return False

    if dry_run:
        print(f"[DRY RUN] Would copy: {source} -> {dest}")
        return True

    # Create destination directory
    dest.parent.mkdir(parents=True, exist_ok=True)

    # Copy file
    shutil.copy2(source, dest)
    print(f"[OK] Copied: {source.name} -> {dest.relative_to(DOCS_PUBLIC)}")
    return True


def copy_papers(dry_run: bool = False) -> int:
    """Copy research papers to public/papers/."""
    print("\n=== Copying Research Papers ===")
    count = 0

    for paper_dir, dest_name in PAPERS:
        source = ARTIFACTS / "papers" / paper_dir / "main.pdf"
        dest = DOCS_PUBLIC / dest_name

        if copy_asset(source, dest, dry_run):
            count += 1

    return count


def copy_figures(dry_run: bool = False) -> int:
    """Copy all figures defined in ASSET_MAP."""
    print("\n=== Copying Figures ===")
    count = 0

    for source, dest, chapter in ASSET_MAP:
        if copy_asset(source, dest, dry_run):
            count += 1

    return count


def generate_manifest(dry_run: bool = False) -> None:
    """Generate JSON manifest of all available assets."""
    print("\n=== Generating Manifest ===")

    manifest: Dict[str, List[Dict[str, str]]] = {}

    for chapter in range(1, 7):
        chapter_key = f"chapter{chapter}"
        manifest[chapter_key] = []

        chapter_dir = DOCS_PUBLIC / "figures" / chapter_key
        if chapter_dir.exists():
            for fig in sorted(chapter_dir.glob("*.png")):
                manifest[chapter_key].append({
                    "filename": fig.name,
                    "path": f"/figures/{chapter_key}/{fig.name}",
                })

    if dry_run:
        print("[DRY RUN] Would generate manifest.json:")
        print(json.dumps(manifest, indent=2))
    else:
        manifest_path = DOCS_PUBLIC / "figures" / "manifest.json"
        manifest_path.parent.mkdir(parents=True, exist_ok=True)

        with open(manifest_path, "w") as f:
            json.dump(manifest, f, indent=2)

        print(f"[OK] Generated: {manifest_path.relative_to(PROJECT_ROOT)}")


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description="Generate educational docs site assets")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be copied without actually copying",
    )
    args = parser.parse_args()

    print("=" * 60)
    print("Educational Docs Site - Asset Preparation")
    print("=" * 60)

    if args.dry_run:
        print("\n[DRY RUN MODE - No files will be modified]\n")

    # Copy assets
    figures_copied = copy_figures(args.dry_run)
    papers_copied = copy_papers(args.dry_run)

    # Generate manifest
    generate_manifest(args.dry_run)

    # Summary
    print("\n" + "=" * 60)
    print("Summary")
    print("=" * 60)
    print(f"Figures copied: {figures_copied}")
    print(f"Papers copied:  {papers_copied}")
    print(f"Total assets:   {figures_copied + papers_copied}")

    if args.dry_run:
        print("\n[DRY RUN] No files were modified. Run without --dry-run to apply changes.")
    else:
        print("\n[DONE] Assets prepared successfully!")
        print("\nNext steps:")
        print("  cd web/docs")
        print("  npm install")
        print("  npm run dev")


if __name__ == "__main__":
    main()
