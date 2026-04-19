@echo off
REM Pipeline Manager - Web UI for managing redistricting runs
REM
REM Usage:
REM   run_manager.bat           Start on default port 5100
REM   run_manager.bat 5101      Start on specific port

setlocal

REM Get port from argument or use default
if "%1"=="" (
    set PORT=5200
) else (
    set PORT=%1
)

echo.
echo ========================================================================
echo Pipeline Manager
echo ========================================================================
echo Starting web server on http://localhost:%PORT%
echo.
echo Press Ctrl+C to stop the server
echo ========================================================================
echo.

REM Kill any existing process on this port
echo Checking for existing server on port %PORT%...
for /f "tokens=5" %%a in ('netstat -aon ^| findstr ":%PORT%" ^| findstr "LISTENING"') do (
    echo Killing existing process %%a on port %PORT%
    taskkill /F /PID %%a >nul 2>&1
)

cd tools\pipeline_manager

REM Start Flask app
python app.py --port %PORT%

if errorlevel 1 (
    echo.
    echo [ERROR] Failed to start server
    echo.
    echo Troubleshooting:
    echo   - Port in use? Try: run_manager.bat 5101
    echo   - Flask not installed? Run: pip install Flask
    echo.
    pause
)

endlocal
