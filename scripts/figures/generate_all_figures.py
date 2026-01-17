#!/usr/bin/env python3
"""
Master script to generate all shared figures.

This script generates all figures used across papers, presentations, and guides,
placing them in the top-level figures/ directory for shared access.
"""

import argparse
import subprocess
import sys
from pathlib import Path
import shutil


def run_command(cmd, description):
    """Run a command and return success status."""
    print(f"  {description}...")
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    if result.returncode == 0:
        print(f"    [OK] {description}")
        return True
    else:
        print(f"    [ERROR] {description} failed:")
        print(result.stderr)
        return False


def generate_schematic_figures(args):
    """Generate schematic/educational figures."""
    print("\n[1/3] Generating schematic figures...")
    print("-" * 70)

    script = Path('outputs/artifacts/presentations/edge_weighted_bisection/create_figures.py')
    if not script.exists():
        print(f"  [ERROR] Script not found: {script}")
        return False

    cmd = f'{sys.executable} {script} --year {args.year} --version {args.version}'
    if not run_command(cmd, "Creating schematic figures"):
        return False

    # Copy to shared figures directory
    source_dir = Path(f'outputs/outputs/artifacts/presentations/edge_weighted_bisection/figures')
    dest_dir = Path('outputs/artifacts/figures/schematic')
    dest_dir.mkdir(parents=True, exist_ok=True)

    schematics = [
        'tract_to_graph.png',
        'graph_with_cut.png',
        'edge_weights_example.png',
        'before_after_cut.png',
        'example_gerrymander.png',
        'real_tracts_to_graph.png'
    ]

    for filename in schematics:
        src = source_dir / filename
        if src.exists():
            shutil.copy2(src, dest_dir / filename)
            print(f"    Copied: {filename}")

    return True


def generate_real_tracts_examples(args):
    """Generate real census tracts examples for appendix."""
    print("\n[2/3] Generating real tracts examples...")
    print("-" * 70)

    script = Path('artifacts/guides/edge_weighted_bisection/create_appendix_examples.py')
    if not script.exists():
        print(f"  [ERROR] Script not found: {script}")
        return False

    cmd = f'{sys.executable} {script} --year {args.year} --version {args.version}'
    if not run_command(cmd, "Creating real tracts examples"):
        return False

    # Copy to shared figures directory
    source_dir = Path('outputs/artifacts/guides/edge_weighted_bisection/appendix_examples')
    dest_dir = Path('outputs/artifacts/figures/real_tracts_examples')
    dest_dir.mkdir(parents=True, exist_ok=True)

    # Copy all PNG files
    if source_dir.exists():
        for png_file in source_dir.glob('*.png'):
            shutil.copy2(png_file, dest_dir / png_file.name)
            print(f"    Copied: {png_file.name}")

    # Generate main text figure variants
    print("\n  Generating main text figure variants...")
    variant_script = Path('scripts/figures/create_figure_variants.py')
    if not variant_script.exists():
        print(f"  [WARNING] Variant script not found: {variant_script}")
        return True  # Not critical, continue

    # Generate specific variants for laymen's guide main text
    # Names auto-generated as: {city}_{ratio}_{panels}_{boundary_labels}_{partition}_{partition_mode}.png
    # Note: Appendix figures handled by create_appendix_examples.py (with ratio in name)
    variants = [
        ('minneapolis_50_50', 'map', 'none', 'before', 'edgeweighted'),         # Figure 1: just geography
        ('minneapolis_50_50', 'both', 'none', 'before', 'edgeweighted'),        # Figure 2: show graph representation before partition
        ('minneapolis_50_50', 'both', 'none', 'after', 'unweighted'),            # Figure 3: show unweighted no labels (50-50)
        ('minneapolis_50_50', 'both', 'all', 'before', 'edgeweighted'),         # Figure 4: show edge weights data before partition
        ('minneapolis_50_50', 'both', 'all', 'after', 'edgeweighted'),          # Figure 5: show edgeweighted cut with all labels (50-50)
        ('minneapolis_67_33', 'both', 'all', 'after', 'edgeweighted'),          # Figure 6: show 2:1 split with edge weights
    ]

    for city, panels, boundary_labels, partition, partition_mode in variants:
        # Let the script auto-generate the filename with all parameters
        # Note: Use 2010 for tract data (that's what we have downloaded)
        cmd = f'{sys.executable} {variant_script} --city {city} --year 2010 --panels {panels} --boundary-labels {boundary_labels} --partition {partition} --partition-mode {partition_mode}'
        # Filename includes ratio extracted from config automatically
        if run_command(cmd, f"Creating figure for {city}"):
            print(f"    Generated figure for {city}")
        else:
            print(f"    [WARNING] Failed to generate figure for {city}")

    return True


def copy_round_progression_maps(args):
    """Copy round progression maps from pipeline outputs."""
    print("\n[3/3] Copying round progression maps...")
    print("-" * 70)

    source_dir = Path(f'outputs/us_{args.year}_{args.version}/states')
    dest_dir = Path('outputs/artifacts/figures/round_progression')
    dest_dir.mkdir(parents=True, exist_ok=True)

    if not source_dir.exists():
        print(f"  [WARNING] Pipeline outputs not found: {source_dir}")
        print(f"  Run: python scripts/pipeline/run_complete_redistricting.py --year {args.year} --version {args.version}")
        return False

    # States and rounds to copy
    states_rounds = [
        ('minnesota', 3),
        ('alabama', 3),
        ('california', 6),
        ('texas', 5)
    ]

    copied = 0
    for state_name, max_round in states_rounds:
        state_dir = source_dir / state_name / 'maps' / 'rounds'
        if not state_dir.exists():
            print(f"  [SKIP] {state_name}: directory not found")
            continue

        for round_num in range(1, max_round + 1):
            round_file = state_dir / f'round_{round_num:02d}.png'
            if round_file.exists():
                dest_file = dest_dir / f'{state_name}_round_{round_num}.png'
                shutil.copy2(round_file, dest_file)
                copied += 1

    if copied > 0:
        print(f"  [OK] Copied {copied} round progression maps")
        return True
    else:
        print(f"  [WARNING] No round progression maps found")
        return False


def create_readme():
    """Create README.md in outputs/artifacts/figures/ directory."""
    readme_content = """# Shared Figures Directory

This directory contains all figures used across papers, presentations, and guides.

## Directory Structure

```
outputs/artifacts/figures/
├── schematic/               # Educational/schematic diagrams
│   ├── tract_to_graph.png               # Shows tract → graph transformation
│   ├── graph_with_cut.png               # Graph with METIS cut visualization
│   ├── edge_weights_example.png         # Edge weight illustration
│   ├── before_after_cut.png             # Before/after comparison
│   ├── example_gerrymander.png          # Classic gerrymandering example
│   └── real_tracts_to_graph.png         # Main example with real data
│
├── real_tracts_examples/    # Real census data examples (various ratios)
│   ├── minneapolis_50_50.png            # Minneapolis (50-50 split)
│   ├── los_angeles_67_33.png            # Los Angeles (67-33 split)
│   ├── houston_60_40.png                # Houston (60-40 split)
│   ├── miami_75_25.png                  # Miami (75-25 split)
│   ├── phoenix_80_20.png                # Phoenix (80-20 split)
│   └── atlanta_55_45.png                # Atlanta (55-45 split)
│
└── round_progression/       # Round-by-round bisection progression
    ├── minnesota_round_1.png
    ├── minnesota_round_2.png
    ├── minnesota_round_3.png
    ├── alabama_round_1.png
    └── ... (state_name_round_N.png)
```

## Usage

### In LaTeX (Papers/Presentations)

```latex
% From papers/
\\includegraphics[width=0.8\\textwidth]{../outputs/artifacts/figures/schematic/tract_to_graph.png}

% From presentations/
\\includegraphics[width=0.8\\textwidth]{../../outputs/artifacts/figures/real_tracts_examples/minneapolis_50_50.png}
```

### Regenerating Figures

To regenerate all figures:

```bash
python scripts/figures/generate_all_figures.py --year 2010 --version v1
```

Or generate specific types:
- Schematic figures: `python outputs/artifacts/presentations/edge_weighted_bisection/create_figures.py`
- Real tracts examples: `python artifacts/guides/edge_weighted_bisection/create_appendix_examples.py`
- Custom variants: `python scripts/figures/create_figure_variants.py --city minneapolis --panels both --boundary-labels all --partition before`

### Creating Custom Figure Variants

The `create_figure_variants.py` script allows flexible generation of educational figures:

**Parameters:**
- `--city`: minneapolis, houston, or los_angeles
- `--panels`: map (geography only), graph (abstract only), or both (side-by-side)
- `--boundary-labels`: all (label all edges), cut (label only cut edges), or none (no labels)
  - Note: Edges/boundaries are always drawn; this parameter only controls labels
- `--partition`: before (original, unpartitioned) or after (partitioned with colors)
- `--year`: Census year (2000, 2010, 2020)

**Examples:**
```bash
# Show transformation with all edge labels visible
python scripts/figures/create_figure_variants.py --city minneapolis --panels both --boundary-labels all --partition before

# Show partition result with cut edge labels highlighted
python scripts/figures/create_figure_variants.py --city minneapolis --panels both --boundary-labels cut --partition after

# Show structure without any labels
python scripts/figures/create_figure_variants.py --city minneapolis --panels map --boundary-labels none --partition before
```

## Figure Descriptions

### Schematic Figures

- **tract_to_graph.png**: Conceptual illustration of how census tracts are converted to a graph representation
- **graph_with_cut.png**: Abstract graph showing METIS cut edges
- **edge_weights_example.png**: Explains how boundary lengths become edge weights
- **before_after_cut.png**: Before/after comparison showing the effect of partitioning
- **example_gerrymander.png**: Classic salamander district example
- **real_tracts_to_graph.png**: Main demonstration with 12 real Minneapolis census tracts

### Real Tracts Examples

Six examples demonstrating METIS with different population ratios across various US cities:
- **50-50**: Equal population split (Minneapolis)
- **67-33**: Supermajority split (Los Angeles)
- **60-40**: Moderate split (Houston)
- **75-25**: Strong majority (Miami)
- **80-20**: Very strong majority (Phoenix)
- **55-45**: Close to equal (Atlanta)

Each figure shows:
- Left panel: Real geographic tracts colored by partition
- Right panel: Abstract graph representation
- Boundary lengths labeled
- Total cut length displayed
- Region population sums

### Round Progression

State-by-state visualization of recursive bisection progress, showing how districts are created through successive splits.

## Notes

- All figures generated at 150 DPI for publication quality
- Census year and version specified at generation time
- Figures are checked into git for easy distribution
- Update figures by rerunning generation script

## Last Generated

Run `python scripts/figures/generate_all_figures.py` to see generation timestamp.
"""

    readme_path = Path('outputs/artifacts/figures/README.md')
    readme_path.parent.mkdir(parents=True, exist_ok=True)
    with open(readme_path, 'w', encoding='utf-8') as f:
        f.write(readme_content)
    print(f"\n[OK] Created: {readme_path}")


def main():
    parser = argparse.ArgumentParser(
        description='Generate all shared figures for papers, presentations, and guides'
    )
    parser.add_argument('--year', type=int, default=2010,
                       choices=[2000, 2010, 2020],
                       help='Census year (default: 2010)')
    parser.add_argument('--version', type=str, default='v1',
                       help='Pipeline version (default: v1)')
    args = parser.parse_args()

    print("=" * 70)
    print("Generating All Shared Figures")
    print("=" * 70)
    print(f"Census year: {args.year}")
    print(f"Version: {args.version}")

    # Change to repository root
    repo_root = Path(__file__).resolve().parents[2]
    import os
    os.chdir(repo_root)
    print(f"Working directory: {repo_root}")

    # Generate all figure types
    success = True
    success &= generate_schematic_figures(args)
    success &= generate_real_tracts_examples(args)
    success &= copy_round_progression_maps(args)

    # Create README
    create_readme()

    print("\n" + "=" * 70)
    if success:
        print("All figures generated successfully!")
    else:
        print("Some figures could not be generated (see warnings above)")
    print("=" * 70)
    print(f"Location: {repo_root / 'outputs' / 'figures'}")
    print()

    return 0 if success else 1


if __name__ == '__main__':
    sys.exit(main())
