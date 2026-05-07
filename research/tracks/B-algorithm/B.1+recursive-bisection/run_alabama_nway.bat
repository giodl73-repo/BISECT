@echo off
REM Test Alabama with direct n-way partitioning (no recursive bisection)

echo ========================================
echo Testing Alabama with Direct 7-way Partitioning
echo ========================================
echo.
echo This bypasses recursive bisection and partitions
echo Alabama into 7 districts in ONE step using tpwgts.
echo.

python scripts/pipeline/test_nway_partition.py --state AL --num-districts 7 --target-mm-districts 2 --year 2020

echo.
echo ========================================
echo Test complete!
echo ========================================
