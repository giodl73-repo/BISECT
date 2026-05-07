# Figure 2: National Redistricting Results

## Requirements
- National map showing all 435 congressional districts from 2020 census
- Color-coded by either:  - Compactness (Polsby-Popper score)  - MM status (majority-minority vs. non-MM)
- Three insets:
  - Alabama: VRA success story (0 enacted → 2 algorithmic MM districts)
  - California: Large state example (52 districts)
  - Vermont: Small state example (1 district)

## Data Source
```
outputs/v1/2020/
├── state_results/  (individual state district assignments)
├── national/       (aggregated national data)
└── maps/           (existing visualizations if available)
```

## Generation Script
Can use existing visualization infrastructure:
```bash
# Option 1: Use existing national map script if available
python scripts/visualization/create_national_map.py --year 2020 --version v1 --output figures/figure2-national-map.pdf

# Option 2: Create custom script for paper
python research/00+synthesis-metapaper/scripts/generate_figure2.py
```

## Output Format
- PDF or high-resolution PNG (300+ DPI for Science)
- Size: Full page width (~7 inches for Science format)
- Include scale bar and legend

## Status
- [ ] Data extraction from pipeline outputs
- [ ] National boundary layer
- [ ] District polygons colored by metric
- [ ] Inset creation (Alabama, California, Vermont)
- [ ] Final rendering at publication quality

## Notes
- Coordinate with main project visualization tools
- May need to run full pipeline if outputs not available
- Consider using existing `/create-national-map` skill for generation
