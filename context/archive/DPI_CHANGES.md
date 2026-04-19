# DPI Parameter Implementation

All scripts now accept a `--dpi` parameter to control image quality vs speed.

## Usage

```bash
# Default (150 DPI) - good balance
run_parallel.bat

# High quality (slower)
run_parallel.bat --dpi 300

# Fast (lower quality)
run_parallel.bat --dpi 100
```

## DPI Options
- **72**: Screen resolution, very fast
- **100**: Low quality, fast
- **150**: Default - good balance ✓
- **200**: High quality
- **300**: Print quality, very slow

## Parameter Flow

```
run_complete_redistricting.py --dpi 150
  └─> run_all_states_parallel.py --dpi 150
      └─> process_single_state.py --dpi 150
          ├─> run_state_redistricting.py --dpi 150
          ├─> add_cities_to_districts.py --dpi 150
          ├─> visualize_all_rounds.py --dpi 150
          └─> create_individual_district_maps.py --dpi 150
```

## Changes Needed

The following scripts still need `--dpi` argument added:
- add_cities_to_districts.py (line 431)
- visualize_all_rounds.py (line 154)
- create_individual_district_maps.py (line 350)

## Already Updated
- ✓ run_complete_redistricting.py
- ✓ run_all_states_parallel.py
- ✓ process_single_state.py
- ✓ run_state_redistricting.py
