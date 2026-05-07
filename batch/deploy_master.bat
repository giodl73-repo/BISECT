@echo off
REM Deploy master dashboard and regenerate all individual dashboards
REM
REM This generates:
REM   1. Landing page at outputs/index.html
REM   2. Run list at outputs/runs.json (used by all individual dashboards)
REM   3. Regenerates ALL individual dashboards with latest template
REM
REM Usage:
REM   deploy_master.bat                  (regenerate everything - default)
REM   deploy_master.bat --skip-dashboards (fast mode - only update master)

REM Change to project root (one level up from batch/)
cd /d "%~dp0\.."

setlocal enabledelayedexpansion

REM Parse arguments
set ARGS=
:parse_args
if "%~1"=="" goto run_script
if /i "%~1"=="--skip-dashboards" (
    set ARGS=%ARGS% --skip-dashboards
)
shift
goto parse_args

:run_script
REM Use py -3.13 to ensure correct Python with installed dependencies
py -3.13 scripts\web\generate_master_dashboard.py %ARGS%

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ========================================
    echo Master dashboard deployment complete!
    echo ========================================
    echo   - Landing page: outputs\index.html
    echo   - Run list: outputs\runs.json
    echo.
    echo Opening outputs\index.html in browser...
    start outputs\index.html
) else (
    echo.
    echo ERROR: Master dashboard generation failed
    exit /b 1
)

endlocal
