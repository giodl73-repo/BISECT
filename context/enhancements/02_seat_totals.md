# E2: Add D/R Seat Totals to Political Maps

**Status**: ✅ COMPLETED
**Priority**: Medium
**Estimated Complexity**: Medium
**Created**: January 2026
**Completed**: January 2026
**Commits**: (Not yet implemented)
**Size**: (Not yet implemented)

### Current State
- Political maps show partisan lean by district (red/blue coloring)
- Created by `scripts/political/visualize_partisan_lean.py`
- No summary statistics shown on map

### Goal
- Add text annotation to political maps showing:
  - Total Democratic-leaning seats (blue ≥ 50%)
  - Total Republican-leaning seats (red > 50%)
  - Format: "D: 27 | R: 25" (example)

### Implementation Plan

#### Files to Modify
1. **`scripts/political/visualize_partisan_lean.py`**
   - Calculate D/R seat counts after assigning colors
   - Add text box annotation to upper-right corner of map
   - Use clean, readable font and contrasting background

#### Changes Required

**visualize_partisan_lean.py:**
```python
# After assigning dem_share and colors to districts
d_seats = (districts_gdf['dem_share'] >= 0.5).sum()
r_seats = (districts_gdf['dem_share'] < 0.5).sum()

# Add text annotation
ax.text(0.98, 0.98, f'D: {d_seats} | R: {r_seats}',
        transform=ax.transAxes,
        fontsize=16,
        fontweight='bold',
        verticalalignment='top',
        horizontalalignment='right',
        bbox=dict(boxstyle='round', facecolor='white', alpha=0.9, edgecolor='black'))
```

#### Output Changes
- Political maps gain seat count annotation
- Example: `california_political.png` shows "D: 42 | R: 10"

#### Benefits
- Quick visual summary of partisan balance
- Easy comparison across states
- Useful for dashboard and paper figures

**Completion Date:** January 11, 2026
**Implementation:** D/R seat count annotation added to `visualize_partisan_lean.py`. All political maps now display seat totals in upper-right corner with readable styling.
