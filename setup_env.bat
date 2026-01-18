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
doskey gco=git checkout $*
doskey gb=git branch $*

REM GitHub CLI shortcuts
doskey ghpr=gh pr create $*
doskey ghprl=gh pr list
doskey ghprv=gh pr view $*
doskey ghprc=gh pr checkout $*
doskey ghprm=gh pr merge $*
doskey ghi=gh issue create $*
doskey ghil=gh issue list
doskey ghiv=gh issue view $*
doskey ghr=gh repo view

REM Project shortcuts - Pipeline
doskey run=run_redistricting.bat $*
doskey runtest=run_test.bat $*
doskey cancel=CANCEL.bat
doskey validate=python scripts/validation/validate_pipeline_outputs.py $*

REM Project shortcuts - Testing
doskey test=pytest tests/ -v
doskey unittest=pytest tests/unit/ -v
doskey inttest=pytest tests/integration/ -v
doskey e2etest=pytest tests/e2e/ -v
doskey dashtest=run_dashboard_tests.bat

REM Project shortcuts - Dashboard
doskey dash=deploy_web.bat $*
doskey gendash=python scripts/web/generate_dashboard.py $*
doskey master=run_master.bat $*

REM Project shortcuts - Data Downloads
doskey dlcensus=python scripts/data/census/download_all_states_tracts.py $*
doskey dltiger2000=python scripts/data/geography/download_tiger_tracts_2000.py $*
doskey dltiger2010=python scripts/data/geography/download_tiger_tracts_2010.py $*
doskey dlplaces=python scripts/data/geography/download_all_places.py $*
doskey dlelect=python scripts/data/elections/download_election_data.py $*
doskey dldemo=python scripts/data/demographics/download_demographic_data_robust.py $*

REM Project shortcuts - Data Processing
doskey buildadj=python scripts/data/geography/build_all_adjacency_graphs.py $*
doskey checkconn=python scripts/data/geography/check_graph_connectivity.py $*

REM Project shortcuts - Artifacts
doskey compile=compile_artifacts.bat

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
echo   Pipeline:
echo     run --version v1                - Run redistricting (multi-year)
echo     runtest --year 2020 --version t - Test run (outputs to dev/)
echo     test                            - Run test suite
echo     dash --year 2020 --version v1   - Deploy dashboard
echo     cancel                          - Cancel running pipeline
echo     compile                         - Compile LaTeX artifacts
echo.
echo   Data Downloads:
echo     dldemo -y 2010                  - Download demographics
echo     dlelect -y 2020                 - Download elections
echo     dlplaces -y 2020                - Download places
echo     dlcensus -y 2020                - Download census tracts
echo     buildadj -y 2020                - Build adjacency graphs
echo.
echo   Git:
echo     gs                              - Git status
echo     ga .                            - Git add all
echo     gc "message"                    - Git commit
echo     gp                              - Git push
echo     gl                              - Git log (10 recent)
echo     gd                              - Git diff
echo.
echo   GitHub CLI:
echo     ghpr                            - Create pull request
echo     ghprl                           - List pull requests
echo     ghprv [number]                  - View pull request
echo     ghprc [number]                  - Checkout pull request
echo     ghi                             - Create issue
echo     ghil                            - List issues
echo.
echo Type 'exit' to close this window
echo ========================================
echo.

REM Keep window open with command prompt
cmd /k
