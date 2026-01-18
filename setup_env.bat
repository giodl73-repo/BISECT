@echo off
REM Congressional Redistricting Development Environment
REM Sets up PATH, environment variables, and window configuration

REM Set window title
title Congressional Redistricting - Development Environment

REM Set window color (navy blue background, bright white text)
color 1F

REM Set window size (120 columns x 40 lines, buffer 120x3000)
mode con: cols=120 lines=40

REM Change to project directory
cd /d "%~dp0"

REM Add batch directory to PATH so shortcuts work
set PATH=%CD%\batch;%PATH%

REM Set Python path (add your Python installation if needed)
REM set PATH=C:\Python313;C:\Python313\Scripts;%PATH%

REM Add project src to PYTHONPATH so imports work
set PYTHONPATH=%CD%\src;%PYTHONPATH%

REM Set project-specific variables
set PROJECT_ROOT=%CD%
set DATA_DIR=%CD%\data
set OUTPUT_DIR=%CD%\outputs
set SCRIPTS_DIR=%CD%\scripts

REM Useful aliases
doskey ls=dir /b $*
doskey ll=dir $*
doskey cat=type $*
doskey clear=cls
doskey grep=findstr $*
doskey ..=cd ..
doskey ...=cd ..\..

REM Git shortcuts
doskey gs=git status
doskey ga=git add $*
doskey gc=git commit -m $*
doskey gp=git push
doskey gl=git log --oneline -10
doskey gd=git diff $*

REM Project shortcuts
doskey run=run_redistricting.bat $*
doskey test=pytest tests/ -v
doskey dash=deploy_web.bat $*

REM Display welcome message
echo.
echo ========================================
echo Congressional Redistricting Dev Env
echo ========================================
echo.
echo Project Root: %PROJECT_ROOT%
echo Python Path:  %PYTHONPATH%
echo.
echo Quick Commands:
echo   run --version v1              - Run pipeline (multi-year)
echo   test                          - Run test suite
echo   dash --year 2020 --version v1 - Deploy dashboard
echo   gs                            - Git status
echo   ga .                          - Git add all
echo   gc "message"                  - Git commit
echo   gp                            - Git push
echo.
echo Type 'exit' to close this window
echo ========================================
echo.

REM Keep window open with command prompt
cmd /k
