# Map Title Updates - Use "Recursive Bifurcation"

## Overview

Update all map titles to use consistent terminology: **"Recursive Bifurcation"** instead of "Algorithmic Redistricting"

This emphasizes the specific algorithm used (recursive bisection) rather than generic "algorithmic" language.

## Files to Update

### 1. National Maps - `scripts/create_us_national_map.py`

**Current titles:**
```python
# Line 223-224 (without cities)
fig.suptitle('United States Congressional Districts - 435 Districts\n2020 Census Algorithmic Redistricting',
             fontsize=22, fontweight='bold', y=0.98)

# Line 401-402 (with cities)
fig.suptitle('United States Congressional Districts - 435 Districts with Cities\n2020 Census Algorithmic Redistricting',
             fontsize=22, fontweight='bold', y=0.98)
```

**Should be:**
```python
# Without cities
fig.suptitle('United States Congressional Districts - 435 Districts\n2020 Census Recursive Bifurcation',
             fontsize=22, fontweight='bold', y=0.98)

# With cities
fig.suptitle('United States Congressional Districts - 435 Districts with Cities\n2020 Census Recursive Bifurcation',
             fontsize=22, fontweight='bold', y=0.98)
```

**With dynamic year support:**
```python
# Get year from output directory or parameter
year = 2020  # Extract from output_dir or add --year parameter

# Without cities
fig.suptitle(f'United States Congressional Districts - 435 Districts\n{year} Census Recursive Bifurcation',
             fontsize=22, fontweight='bold', y=0.98)

# With cities
fig.suptitle(f'United States Congressional Districts - 435 Districts with Cities\n{year} Census Recursive Bifurcation',
             fontsize=22, fontweight='bold', y=0.98)
```

### 2. State Maps - Already Correct! ✅

`scripts/visualize_districts.py` already uses correct terminology:
```python
# Line 97
title = f"{args.state.upper()} {args.num_districts} Congressional Districts\nRecursive Bifurcation Algorithm"
```

### 3. Round Visualizations - `scripts/visualize_all_rounds.py`

**Current titles:**
```python
# Line 164-169
title = f'{state_name} Round {round_num}: {num_regions} Regions\\n'
if num_regions > 1:
    dprs = metadata['total_districts'] // num_regions
    if metadata['total_districts'] % num_regions == 0:
        title += f'({dprs}-{dprs + 1} districts each)'
    else:
        title += f'Tract-Level Redistricting'
```

**Consider updating to:**
```python
title = f'{state_name} Round {round_num}: {num_regions} Regions\\n'
title += 'Recursive Bifurcation'
if num_regions > 1:
    dprs = metadata['total_districts'] // num_regions
    if metadata['total_districts'] % num_regions == 0:
        title += f' - ({dprs}-{dprs + 1} districts each)'
```

### 4. Markdown Reports - Consider Updating

**Files:**
- `outputs/us_2020_v2/US_2020_Redistricting_Report.md`
- Template in `scripts/create_us_aggregate.py`

**Current methodology section:**
```markdown
- **Algorithm**: Recursive bisection using METIS graph partitioning
```

**Could emphasize:**
```markdown
- **Algorithm**: Recursive Bifurcation using METIS graph partitioning
- **Method**: Hierarchical bisection (divide-and-conquer)
```

## Implementation Priority

1. **HIGH**: National maps (`create_us_national_map.py`) - User specifically requested
2. **MEDIUM**: Round visualizations (`visualize_all_rounds.py`) - For consistency
3. **LOW**: Markdown reports - Nice to have, not critical
4. **DONE**: State district maps already correct ✅

## Dynamic Year Support

When updating, also add year parameter support:

```python
def main(output_dir=None, year=2020):
    """Create national US maps."""

    # Use year in titles
    title = f'United States Congressional Districts - 435 Districts\n{year} Census Recursive Bifurcation'
```

This way titles automatically update for 2010 and 2000 census data.

## Testing

After updates, regenerate maps for v2:
```bash
python scripts/create_us_national_map.py --output-dir outputs/us_2020_v2 --year 2020
```

Verify both national maps have updated titles:
- `US_National_Map_435_Districts.png`
- `US_National_Map_435_Districts_With_Cities.png`

---

**Status**: Added to TODO list
**Priority**: User requested
**Estimated Time**: 10-15 minutes
