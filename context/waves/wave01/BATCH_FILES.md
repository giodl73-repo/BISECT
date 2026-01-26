# Batch File Quick Reference

## Running Redistricting

### Parallel Mode (Fast - 4-8 states at once)
```bash
run_parallel.bat
run_parallel.bat --workers 8
run_parallel.bat --version v2 --workers 6
run_parallel.bat --year 2010
```

### Sequential Mode (One state at a time)
```bash
run_sequential.bat
run_sequential.bat --version v2
run_sequential.bat --year 2010
```

### Full Control
```bash
run_redistricting.bat --mode parallel --workers 4 --version v1 --year 2020
```

## Available Parameters

- `--mode sequential|parallel` - Execution mode (default: sequential)
- `--workers N` - Number of parallel workers (default: 4, max: 8)
- `--dpi N` - Map quality: 100 (fast), 150 (default), 200 (high), 300 (print)
- `--version NAME` - Version identifier (default: v1)
- `--year 2020|2010` - Census year (default: 2020)
- `--output-dir PATH` - Custom output directory
- `--print-only` - Dry run (show what would happen)
- `--debug` - Enable debug output
- `--skip-states` - Skip state processing (post-processing only)

## Canceling

**During execution:**
- Press `Ctrl+C` (may need to press 2-3 times)

**If stuck:**
- Run `CANCEL.bat` - kills all Python processes (nuclear option)
- Or open Task Manager and end Python processes manually

## Examples

**Test run (dry run):**
```bash
run_parallel.bat --print-only
```

**Production run with 6 workers and high quality:**
```bash
run_parallel.bat --workers 6 --dpi 200 --version v3
```

**Fast run with lower quality (good for testing):**
```bash
run_parallel.bat --dpi 100 --workers 8
```

**2010 Census data:**
```bash
run_parallel.bat --year 2010 --version v1_2010
```

## Performance Tips

- **DPI 100-150**: Good for testing/iteration (2-4x faster)
- **DPI 200-300**: For final production maps
- **Workers 4-6**: Safe on most systems
- **Workers 8**: High-end systems only

**Process only specific states:**
```bash
python scripts/run_all_states_parallel.py --version test CA TX FL
```
