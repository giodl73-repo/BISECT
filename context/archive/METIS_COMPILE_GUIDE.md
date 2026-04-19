# METIS Windows Compilation Guide

## The Problem

METIS 5.2.1 has Windows compilation issues:
1. CMakeLists.txt references `build/xinclude` which doesn't exist
2. GKlib needs to be in the right place
3. POSIX headers (regex.h) missing on Windows

## Quick Solution Options

### Option 1: Use Conda (Easiest - 5 minutes)

```bash
# Install conda/miniconda if you don't have it
conda create -n redistricting python=3.10
conda activate redistricting
conda install -c conda-forge metis
pip install pymetis

# Test
python -c "import pymetis; print('Success!')"
```

### Option 2: Manual Compilation (30-60 minutes)

#### Step 1: Fix CMakeLists.txt

The issue is line 50 in `CMakeLists.txt`:
```cmake
add_subdirectory("build/xinclude")  # This fails!
```

**Fix:** Comment it out or create dummy directory:
```bash
cd C:\Users\giodl\sources\repo\Apportionment\tools\METIS-5.2.1
mkdir -p build\xinclude
echo "# Dummy" > build\xinclude\CMakeLists.txt
```

#### Step 2: Build with Visual Studio

Open **Visual Studio 2022 Developer Command Prompt** (not regular cmd!):

```batch
cd C:\Users\giodl\sources\repo\Apportionment\tools\METIS-5.2.1
mkdir build-vs
cd build-vs

REM Configure
cmake .. -G "Visual Studio 17 2022" -A x64 ^
  -DGKLIB_PATH=C:\Users\giodl\sources\repo\Apportionment\tools\METIS-5.2.1\GKlib ^
  -DCMAKE_INSTALL_PREFIX=C:\metis

REM Build
cmake --build . --config Release

REM Find gpmetis.exe
dir /s gpmetis.exe
```

Look for: `build-vs\programs\Release\gpmetis.exe`

#### Step 3: Test

```bash
# Copy to a location in PATH or test directly
C:\Users\giodl\sources\repo\Apportionment\tools\METIS-5.2.1\build-vs\programs\Release\gpmetis.exe

# Should print usage info if it works
```

### Option 3: Use Pre-Built Binary

Download from: https://github.com/KarypisLab/METIS/releases (if available)

Or use the one from conda-forge.

## Integration with Our System

Once gpmetis.exe is built, our system will auto-detect it in:

1. System PATH
2. `C:\metis\bin\gpmetis.exe`
3. `C:\Users\giodl\sources\repo\Apportionment\tools\METIS-5.2.1\build-vs\programs\Release\gpmetis.exe`

Test detection:
```bash
python -c "from src.apportionment.partition.metis_executable import test_gpmetis_installation; test_gpmetis_installation()"
```

## If All Else Fails

The NetworkX fallback will work (just slower and lower quality):
```bash
python scripts/redistrict_ca.py --state CA --num-districts 52 --yes
# Will use NetworkX automatically if METIS not found
```

## Common Errors

### Error: `regex.h not found`
**Cause:** GKlib uses POSIX regex
**Fix:** Use conda-forge build OR comment out regex code in GKlib (not recommended)

### Error: `build/xinclude directory not found`
**Cause:** CMakeLists.txt bug
**Fix:** Create dummy directory (see Step 1 above)

### Error: `GKLIB_PATH not found`
**Cause:** GKlib not copied to METIS directory
**Fix:** `cp -r GKlib-master METIS-5.2.1/GKlib`

## What We've Done Already

✓ Fixed CMakeLists.txt version (3.5)
✓ Copied GKlib to METIS directory
✓ Created direct gpmetis.exe wrapper in our code

## Recommendation

**Use Conda** - it's pre-compiled and tested for Windows. Save yourself the headache!

```bash
conda install -c conda-forge metis
pip install pymetis
```

Done!
