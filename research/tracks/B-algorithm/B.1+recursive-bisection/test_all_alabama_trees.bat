@echo off
REM Test all 6 tree structures on Alabama

echo ========================================
echo Testing all tree structures on Alabama
echo ========================================
echo.

set SPLITS=6,1 5,2 4,3 3,4 2,5 1,6

for %%s in (%SPLITS%) do (
    echo.
    echo ========================================
    echo Testing tree structure: 7 -^> [%%s]
    echo ========================================
    python scripts/pipeline/run_state_redistricting.py --state AL --partition-mode metis-vra --target-mm-districts 2 --tree-structure %%s --year 2020 --version V1 --output outputs/test_vra_alabama_tree_%%s 2>&1 | findstr /C:"[VRA]" /C:"District"
    echo.
)

echo.
echo ========================================
echo All tests complete!
echo ========================================
