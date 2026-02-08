@echo off
REM Cross-Census Validation Experiment Launcher
REM Runs in background and logs to file

echo Starting cross-census validation experiment...
echo This will run for approximately 8-12 hours
echo.
echo Log file: research\gerry-cross-census-validation\validation_experiment.log
echo Results: research\gerry-cross-census-validation\results\
echo.

REM Create results directory
mkdir research\gerry-cross-census-validation\results 2>nul

REM Option 1: Quick proof-of-concept (2 hours) - 5 states, 2020 only
echo [Option 1] Quick validation (5 states, 2020 only) - 2 hours
echo python scripts/pipeline/run_validation_quick.py
echo.

REM Option 2: Full validation (12 hours) - 50 states, all years
echo [Option 2] Full validation (50 states, 3 years) - 12 hours
echo python scripts/pipeline/run_cross_census_validation.py --years 2000 2010 2020
echo.

set /p CHOICE="Enter choice (1 or 2): "

if "%CHOICE%"=="1" (
    echo Running quick validation...
    start /b python scripts/pipeline/run_validation_quick.py > research\gerry-cross-census-validation\validation_experiment.log 2>&1
) else if "%CHOICE%"=="2" (
    echo Running full validation...
    start /b python scripts/pipeline/run_cross_census_validation.py --years 2000 2010 2020 > research\gerry-cross-census-validation\validation_experiment.log 2>&1
) else (
    echo Invalid choice
    exit /b 1
)

echo.
echo Experiment started in background!
echo Monitor progress: tail -f research\gerry-cross-census-validation\validation_experiment.log
echo.
