@echo off
REM Test all approaches with ubvec=1000 for comparison

echo ========================================
echo Testing All Approaches with ubvec=1000
echo ========================================
echo.

echo 1. Recursive [3,4] + ubvec
echo ----------------------------
python scripts/pipeline/run_state_redistricting.py --state AL --partition-mode metis-vra --target-mm-districts 2 --tree-structure 3,4 --year 2020 --version test_ubvec_3_4 --output outputs/test_vra_alabama_ubvec_3_4

echo.
echo 2. Recursive [4,3] + ubvec  
echo ----------------------------
python scripts/pipeline/run_state_redistricting.py --state AL --partition-mode metis-vra --target-mm-districts 2 --tree-structure 4,3 --year 2020 --version test_ubvec_4_3 --output outputs/test_vra_alabama_ubvec_4_3

echo.
echo ========================================
echo Tests complete! Check outputs for results.
echo ========================================
