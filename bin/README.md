# METIS Binary Distribution

This directory contains the METIS graph partitioning executable and its dependencies for Windows.

## Files

- **gpmetis.exe** - METIS graph partitioning program (version 5.2.1)
- **libsystre-0.dll** - System library dependency
- **libwinpthread-1.dll** - Threading library dependency

## Usage

The redistricting scripts will automatically find and use this executable. No installation required.

## Source

METIS is developed by George Karypis and the University of Minnesota.
- Website: http://glaros.dtc.umn.edu/gkhome/metis/metis/overview
- License: Apache License 2.0

## Platform

These binaries are for Windows (x64). For other platforms, you'll need to:
1. Download METIS source from the official website
2. Compile it for your platform
3. Either add to PATH or place in this bin/ directory
