@echo off
REM Deploy web dashboard to outputs directory
REM
REM Usage:
REM   deploy_web.bat                                    (defaults to 2020 v1)
REM   deploy_web.bat --year 2020 --version v2
REM   deploy_web.bat --year 2020 --version v1 --output-dir outputs/custom

setlocal enabledelayedexpansion

REM Default values
set YEAR=2020
set VERSION=v1
set OUTPUT_DIR=

REM Parse arguments
:parse_args
if "%~1"=="" goto check_args
if /i "%~1"=="--year" (
    set YEAR=%~2
    shift
    shift
    goto parse_args
)
if /i "%~1"=="--version" (
    set VERSION=%~2
    shift
    shift
    goto parse_args
)
if /i "%~1"=="--output-dir" (
    set OUTPUT_DIR=%~2
    shift
    shift
    goto parse_args
)
shift
goto parse_args

:check_args
REM Year and version now have defaults, so no validation needed

REM Build command
set CMD=python scripts\web\generate_dashboard.py --year %YEAR% --version %VERSION%
if not "%OUTPUT_DIR%"=="" (
    set CMD=%CMD% --output-dir %OUTPUT_DIR%
)

echo Generating dashboard for %YEAR% Census, version %VERSION%...
echo.

%CMD%

if %ERRORLEVEL% EQU 0 (
    echo.
    echo Dashboard generated successfully!

    REM Determine output file location
    if "%OUTPUT_DIR%"=="" (
        set OUTPUT_FILE=outputs\us_%YEAR%_%VERSION%\index.html
    ) else (
        set OUTPUT_FILE=%OUTPUT_DIR%\index.html
    )

    echo Opening !OUTPUT_FILE! in browser...
    start !OUTPUT_FILE!
) else (
    echo.
    echo ERROR: Dashboard generation failed
    exit /b 1
)

endlocal
