@echo off
REM Enhancement Manager Launcher
REM Starts the Flask server and opens browser

REM Change to the script's directory
cd /d "%~dp0"

echo.
echo ========================================
echo   Enhancement Manager
echo ========================================
echo.

REM Check if requirements are installed
py -3.13 -c "import flask" 2>nul
if errorlevel 1 (
    echo [WARN] Flask not found. Installing dependencies...
    py -3.13 -m pip install -r requirements.txt
    if errorlevel 1 (
        echo [FAIL] Failed to install dependencies
        pause
        exit /b 1
    )
)

REM Check if port 5001 is already in use and kill the process
netstat -ano | findstr :5001 | findstr LISTENING >nul
if not errorlevel 1 (
    echo [WARN] Port 5001 is already in use
    echo [OK] Finding and killing existing server...

    REM Get the PID of the process using port 5001
    for /f "tokens=5" %%a in ('netstat -ano ^| findstr :5001 ^| findstr LISTENING') do (
        echo [OK] Killing process %%a...
        taskkill //F //PID %%a >nul 2>&1
    )

    REM Wait a moment for the port to be released
    timeout /t 2 /nobreak >nul
    echo [OK] Port 5001 is now free
    echo.
)

REM Start the Flask server
echo [OK] Starting Enhancement Manager...
echo [OK] Server will run at http://localhost:5001
echo.

REM Start Flask server in background
start /B py -3.13 app.py

REM Wait for server to fully start (5 seconds)
echo [OK] Waiting for server to start...
ping 127.0.0.1 -n 6 >nul

REM Open browser
echo [OK] Opening browser...
start "" http://localhost:5001

echo.
echo Press Ctrl+C to stop the server
echo.

pause
