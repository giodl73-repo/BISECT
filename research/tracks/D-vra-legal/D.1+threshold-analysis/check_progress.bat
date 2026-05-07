@echo off
REM Quick progress checker for 50-state analysis

echo ============================================
echo 50-STATE ANALYSIS PROGRESS
echo ============================================
echo.

REM Check if running
tasklist /FI "PID eq %1" 2>NUL | find /I /N "python">NUL
if "%ERRORLEVEL%"=="0" (
    echo Status: RUNNING
) else (
    echo Status: COMPLETED or NOT STARTED
)

echo.
echo Latest progress:
tail -30 50_state_run.log 2>NUL

echo.
echo ============================================
echo Monitor live: tail -f 50_state_run.log
echo ============================================
