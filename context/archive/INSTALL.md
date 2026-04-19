# Installation Guide

## Quick Start (Linux/macOS)

```bash
# Install dependencies
pip install -r requirements.txt

# That's it! You're ready to go.
```

## Windows Installation

### Prerequisites

- Python 3.10+
- Visual Studio 2019 or later (for METIS compilation)
- CMake 3.15+

### Step 1: Install Python Dependencies

```bash
pip install numpy pandas geopandas shapely pyarrow pyogrio libpysal pyproj pygris networkx matplotlib tqdm
```

### Step 2: Compile and Install METIS

**Option A: Using Conda (Easiest)**

```bash
conda install -c conda-forge metis
pip install pymetis
```

**Option B: Manual Compilation**

pymetis includes METIS source code and attempts to compile it automatically. However, Windows compilation often fails due to missing dependencies (regex.h, etc.).

To compile manually:

1. Clone METIS source:
   ```bash
   git clone https://github.com/KarypisLab/METIS.git
   cd METIS
   ```

2. Follow Windows build instructions in `BUILD-Windows.txt`:
   ```bash
   mkdir build
   cd build
   cmake -G "Visual Studio 17 2022" -A x64 ..
   cmake --build . --config Release
   ```

3. Add METIS to system PATH

4. Install pymetis:
   ```bash
   pip install pymetis
   ```

**Option C: Use MSYS2 (Recommended for Windows)**

```bash
# In MSYS2 terminal:
pacman -S mingw-w64-x86_64-metis
pip install pymetis
```

### Troubleshooting

**Error: `regex.h` not found**

This is a common Windows compilation error. Solutions:
- Use conda-forge (Option A)
- Use MSYS2 with MinGW (Option C)
- Install POSIX regex library for Visual Studio

**Error: `error C2059: syntax error`**

METIS GKlib has compatibility issues with newer Visual Studio versions. Try:
- Using conda-forge prebuilt binaries
- Applying patches from pymetis GitHub issues

## Fallback Option (No METIS)

The system includes a NetworkX fallback that works without METIS:

```bash
pip install -r requirements.txt
# Skip METIS installation
```

**Note:** NetworkX Kernighan-Lin bisection is slower and produces lower-quality partitions than METIS. It's suitable for small tests but not recommended for full-state redistricting.

## Verifying Installation

```bash
# Test METIS installation
python -c "import pymetis; print('PyMetis installed successfully')"

# Or check what partitioning method is available
python -c "from src.apportionment.partition.metis_wrapper import check_metis_installation; print(check_metis_installation())"
```

## Census API Key (Optional)

For automatic population data download, set your Census API key:

```bash
# Get free API key from: https://api.census.gov/data/key_signup.html
export CENSUS_API_KEY="your_key_here"  # Linux/macOS
set CENSUS_API_KEY=your_key_here       # Windows CMD
$env:CENSUS_API_KEY="your_key_here"    # Windows PowerShell
```

Without an API key, you'll need to manually add population data (see `scripts/add_mock_population.py`).

## Next Steps

See README.md for usage examples.

## Known Issues on Windows

1. **pymetis compilation fails** - Use conda-forge or MSYS2
2. **pygris download slow** - Normal; Census shapefiles are large
3. **Adjacency computation slow** - Expected for ~700k blocks; takes 30-60 minutes for full California

## Sources

- [METIS on GitHub](https://github.com/KarypisLab/METIS)
- [METIS Windows Build Instructions](https://github.com/KarypisLab/METIS/blob/master/BUILD-Windows.txt)
- [PyMetis on PyPI](https://pypi.org/project/pymetis/)
- [Conda-forge PyMetis](https://anaconda.org/conda-forge/pymetis)
