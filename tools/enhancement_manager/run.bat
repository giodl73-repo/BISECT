@echo off
REM Enhancement Manager Launcher
REM Starts the Flask server and opens browser

echo.
echo ========================================
echo   Enhancement Manager
echo ========================================
echo.

REM Check if requirements are installed
python -c "import flask" 2>nul
if errorlevel 1 (
    echo [WARN] Flask not found. Installing dependencies...
    pip install -r requirements.txt
    if errorlevel 1 (
        echo [FAIL] Failed to install dependencies
        pause
        exit /b 1
    )
)

REM Check if port 5000 is already in use
netstat -ano | findstr :5000 | findstr LISTENING >nul
if not errorlevel 1 (
    echo [WARN] Port 5000 is already in use
    echo.
    echo Another Flask instance may be running.
    echo Please close it or change the port in app.py
    echo.
    pause
    exit /b 1
)

REM Start the Flask server
echo [OK] Starting Enhancement Manager...
echo [OK] Server will run at http://localhost:5000
echo.
echo Press Ctrl+C to stop the server
echo.

REM Wait a moment then open browser
start "" http://localhost:5000

REM Start Flask server
python app.py

pause
