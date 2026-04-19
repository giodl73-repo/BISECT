# Dependencies

Installation guide for all project dependencies.

## Overview

This project requires:
1. **Python 3.8+** with geospatial packages
2. **METIS** graph partitioning library
3. **Census API key** (optional, for data downloads)

## System Requirements

### Minimum
- Python 3.8+
- 8 GB RAM
- 20 GB disk space
- Windows, Linux, or macOS

### Recommended
- Python 3.10+
- 16 GB RAM (for parallel processing)
- 30 GB disk space
- 4-8 CPU cores

## Python Environment Setup

### Using Conda (Recommended)

Conda provides pre-compiled packages for geospatial dependencies, which can be difficult to install via pip on Windows.

```bash
# Create environment
conda create -n redistricting python=3.10
conda activate redistricting

# Install geospatial packages from conda-forge
conda install -c conda-forge geopandas
conda install -c conda-forge pyarrow
conda install -c conda-forge networkx
conda install -c conda-forge matplotlib
conda install -c conda-forge tqdm

# Install METIS (optional but recommended)
conda install -c conda-forge metis

# Install remaining packages via pip
pip install requests
pip install pymetis
```

### Using pip Only

```bash
# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install packages
pip install geopandas
pip install pyarrow
pip install networkx
pip install matplotlib
pip install tqdm
pip install requests
pip install pymetis
```

**Note**: Installing geopandas via pip can be problematic on Windows due to GDAL/GEOS dependencies. Use conda if you encounter issues.

## Core Dependencies

### Geospatial Libraries

**geopandas** (>=0.12.0)
- Geospatial data processing
- Used for: Tract geometries, adjacency calculations
- Includes: shapely, fiona, pyproj

**pyarrow** (>=10.0.0) or **fastparquet**
- Parquet file I/O
- Used for: All tract, demographic, and election data
- Recommendation: pyarrow (faster, better compatibility)

### Graph Libraries

**networkx** (>=2.8)
- Graph data structures and algorithms
- Used for: Adjacency graphs, contiguity checks
- Required for: All redistricting operations

### Visualization

**matplotlib** (>=3.5.0)
- Static map generation
- Used for: All district maps

**contextily** (optional)
- Basemap tiles for maps
- Not currently used but useful for web-style maps

### Utilities

**tqdm** (>=4.65.0)
- Progress bars
- Used for: All long-running operations

**requests** (>=2.28.0)
- HTTP requests
- Used for: Census API calls, data downloads

## METIS Installation

METIS is a high-performance graph partitioning library. It's optional but highly recommended for better quality districts.

### Why METIS?

- **Better Quality**: Produces more compact districts
- **Faster**: O(|E| log k) vs O(k × |E|) for alternatives
- **Industry Standard**: Used in circuit design, FEM, distributed computing

### Installation Options

#### Option 1: Conda (Easiest - 5 minutes) ⭐ RECOMMENDED

```bash
conda install -c conda-forge metis
pip install pymetis

# Test
python -c "import pymetis; print('METIS installed successfully!')"
```

#### Option 2: Using Pre-Built Binaries

**Windows**:
1. Download METIS from https://github.com/KarypisLab/METIS/releases
2. Extract to `C:\metis\`
3. Add `C:\metis\bin` to PATH
4. Test: `gpmetis.exe` (should show usage)

**Linux**:
```bash
# Ubuntu/Debian
sudo apt-get install metis libmetis-dev

# RHEL/CentOS
sudo yum install metis metis-devel

# Test
which gpmetis
```

**macOS**:
```bash
brew install metis

# Test
which gpmetis
```

#### Option 3: Compile from Source (Advanced - 30-60 minutes)

See [METIS Compilation Guide](#metis-compilation-guide) below.

### Verification

Test METIS installation:

```python
from src.apportionment.partition import metis_wrapper

# Try to import pymetis
try:
    import pymetis
    print("✓ pymetis library found")
except ImportError:
    print("✗ pymetis not installed (fallback to executable)")

# Try to find gpmetis executable
from src.apportionment.partition.metis_executable import find_gpmetis
gpmetis_path = find_gpmetis()
if gpmetis_path:
    print(f"✓ gpmetis found at: {gpmetis_path}")
else:
    print("✗ gpmetis not found")
```

### Fallback: NetworkX

If METIS is not available, the system will automatically fall back to NetworkX graph partitioning:

```python
# src/apportionment/partition/metis_wrapper.py
try:
    import pymetis
    USE_METIS = True
except ImportError:
    USE_METIS = False
    print("Warning: pymetis not found, using NetworkX fallback (slower)")
```

**Trade-offs**:
- ✓ No external dependencies
- ✓ Pure Python
- ✗ Slower (~3-5x)
- ✗ Lower quality partitions

## Census API Key (Optional)

Required only for downloading new data. Not needed if using pre-downloaded data.

### Get API Key

1. Visit: https://api.census.gov/data/key_signup.html
2. Fill out form with email
3. Receive key via email (instant)

### Configure API Key

**Option 1: Environment Variable** (Recommended)
```bash
# Linux/macOS
export CENSUS_API_KEY="your_key_here"

# Windows (PowerShell)
$env:CENSUS_API_KEY="your_key_here"

# Windows (cmd)
set CENSUS_API_KEY=your_key_here
```

**Option 2: Configuration File**
```python
# config.py
CENSUS_API_KEY = "your_key_here"
```

**Option 3: Command Line**
```bash
python scripts/data/demographics/download_demographic_data_robust.py --api-key YOUR_KEY
```

## Verification Script

Test all dependencies:

```python
#!/usr/bin/env python3
"""Verify all dependencies are installed."""

import sys

def check_import(module_name, display_name=None):
    """Try to import module and report status."""
    if display_name is None:
        display_name = module_name

    try:
        __import__(module_name)
        print(f"✓ {display_name}")
        return True
    except ImportError:
        print(f"✗ {display_name} (not installed)")
        return False

print("Checking dependencies...\n")

# Core packages
check_import("geopandas", "geopandas (geospatial)")
check_import("pandas", "pandas (dataframes)")
check_import("numpy", "numpy (numerical)")

# File I/O
has_pyarrow = check_import("pyarrow", "pyarrow (parquet)")
has_fastparquet = check_import("fastparquet", "fastparquet (parquet, alternative)")
if not (has_pyarrow or has_fastparquet):
    print("  ⚠ Warning: Need pyarrow OR fastparquet for parquet files")

# Graph processing
check_import("networkx", "networkx (graphs)")

# Visualization
check_import("matplotlib", "matplotlib (plotting)")
check_import("matplotlib.pyplot", "matplotlib.pyplot")

# Utilities
check_import("tqdm", "tqdm (progress bars)")
check_import("requests", "requests (HTTP)")

# METIS (optional)
print("\nOptional dependencies:")
has_pymetis = check_import("pymetis", "pymetis (METIS library)")

# Check for gpmetis executable
import shutil
gpmetis = shutil.which("gpmetis")
if gpmetis:
    print(f"✓ gpmetis executable (found at: {gpmetis})")
elif not has_pymetis:
    print("✗ gpmetis executable (not in PATH)")
    print("  ⚠ Warning: No METIS found - will use NetworkX fallback (slower)")

# Check Census API key
import os
api_key = os.environ.get("CENSUS_API_KEY")
if api_key:
    print(f"✓ Census API key (configured)")
else:
    print("✗ Census API key (not set)")
    print("  ⓘ Info: Only needed for downloading new data")

print("\nDependency check complete!")
```

Save as `scripts/check_dependencies.py` and run:

```bash
python scripts/check_dependencies.py
```

## METIS Compilation Guide

### Windows Compilation (Advanced)

METIS 5.2.1 has Windows compilation issues. Conda installation is strongly recommended. If you must compile from source:

#### Prerequisites
- Visual Studio 2019 or 2022 (Community Edition is fine)
- CMake 3.15+
- Git

#### Known Issues
1. CMakeLists.txt references `build/xinclude` which doesn't exist
2. GKlib must be in correct location
3. POSIX headers (regex.h) missing on Windows

#### Compilation Steps

**Step 1: Get Source Code**
```bash
git clone https://github.com/KarypisLab/METIS.git
cd METIS
git checkout v5.2.1
```

**Step 2: Fix CMakeLists.txt**

The issue is line 50 in `CMakeLists.txt`:
```cmake
add_subdirectory("build/xinclude")  # This fails!
```

**Fix:** Create dummy directory:
```bash
mkdir -p build\xinclude
echo "# Dummy" > build\xinclude\CMakeLists.txt
```

**Step 3: Get GKlib**
```bash
git clone https://github.com/KarypisLab/GKlib.git
# GKlib should be in METIS/GKlib/ directory
```

**Step 4: Build with Visual Studio**

Open **Visual Studio Developer Command Prompt** (not regular cmd!):

```batch
cd METIS
mkdir build-vs
cd build-vs

:: Configure
cmake .. -G "Visual Studio 17 2022" -A x64 ^
  -DGKLIB_PATH=%cd%\..\GKlib ^
  -DCMAKE_INSTALL_PREFIX=C:\metis

:: Build
cmake --build . --config Release

:: Find gpmetis.exe
dir /s gpmetis.exe
```

**Step 5: Install**
```batch
:: Copy to install location
cmake --install . --config Release

:: Or manually copy
copy programs\Release\gpmetis.exe C:\metis\bin\
copy programs\Release\mpmetis.exe C:\metis\bin\

:: Add to PATH
setx PATH "%PATH%;C:\metis\bin"
```

**Step 6: Test**
```bash
gpmetis.exe
# Should print usage information
```

#### Common Errors

**Error: `regex.h not found`**
- **Cause**: GKlib uses POSIX regex, not available on Windows
- **Fix**: Use conda-forge build (has Windows compatibility patches)

**Error: `build/xinclude directory not found`**
- **Cause**: CMakeLists.txt bug in METIS 5.2.1
- **Fix**: Create dummy directory (see Step 2)

**Error: `GKLIB_PATH not found`**
- **Cause**: GKlib not in expected location
- **Fix**: Ensure GKlib is in `METIS/GKlib/` directory

### Linux Compilation

Much easier on Linux due to POSIX compatibility:

```bash
# Install dependencies
sudo apt-get install build-essential cmake

# Download and compile
wget http://glaros.dtc.umn.edu/gkhome/fetch/sw/metis/metis-5.2.1.tar.gz
tar -xzf metis-5.2.1.tar.gz
cd metis-5.2.1

# Build
make config prefix=/usr/local
make
sudo make install

# Test
gpmetis
```

### macOS Compilation

```bash
# Install dependencies
brew install cmake

# Download and compile (same as Linux)
wget http://glaros.dtc.umn.edu/gkhome/fetch/sw/metis/metis-5.2.1.tar.gz
tar -xzf metis-5.2.1.tar.gz
cd metis-5.2.1

# Build
make config prefix=/usr/local
make
sudo make install

# Test
gpmetis
```

## Project-Specific Setup

After installing dependencies:

1. **Verify installation**
```bash
python scripts/check_dependencies.py
```

2. **Download initial data** (optional)
```bash
# Download California tracts (test case)
python scripts/data/census/download_all_states_tracts.py --year 2020 --states CA

# Build adjacency graph
python scripts/data/geography/build_adjacency_graphs.py --state CA --year 2020
```

3. **Run test redistricting**
```bash
# Process Vermont (1 district, simple test)
python scripts/pipeline/run_state_redistricting.py --state VT --year 2020 --output-dir outputs/test/vermont
```

4. **Verify output**
```bash
ls outputs/test/vermont/
# Should see: final_assignments.pkl, district_summary.csv, maps/, etc.
```

## Troubleshooting

### Import Errors

**Problem**: `ImportError: cannot import name 'X' from 'Y'`

**Solution**: Ensure you're using compatible versions:
```bash
pip list | grep -E 'geopandas|pandas|numpy|networkx'
```

### GDAL/GEOS Errors (Windows)

**Problem**: `OSError: Could not find libgdal or GDAL installation`

**Solution**: Use conda instead of pip:
```bash
conda install -c conda-forge geopandas
```

### Memory Errors

**Problem**: `MemoryError` during large state processing

**Solution**:
1. Increase system RAM (recommend 16 GB)
2. Close other applications
3. Process states sequentially instead of parallel:
```bash
python scripts/pipeline/run_complete_redistricting.py --workers 1
```

### Slow Performance

**Problem**: Redistricting very slow (>2 hours per state)

**Possible Causes**:
1. METIS not installed (using NetworkX fallback)
2. DPI set to 300 (use 150 instead)
3. Low CPU count

**Solutions**:
```bash
# Install METIS
conda install -c conda-forge metis

# Use lower DPI
--dpi 150

# Use more workers
--workers 4
```

## Version Requirements Summary

### Required
```
python>=3.8
geopandas>=0.12.0
pandas>=1.4.0
numpy>=1.22.0
networkx>=2.8
matplotlib>=3.5.0
pyarrow>=10.0.0
tqdm>=4.65.0
requests>=2.28.0
```

### Optional
```
metis>=5.2.0
pymetis>=2023.1
contextily>=1.3.0
```

### Development (Optional)
```
pytest>=7.0.0
black>=22.0.0
flake8>=4.0.0
mypy>=0.990
```

## See Also

- `CODING_PATTERNS.md` - Coding conventions
- `ARCHITECTURE.md` - System design
- `scripts/data/README.md` - Data acquisition
