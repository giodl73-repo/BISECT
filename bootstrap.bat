@echo off
REM bootstrap.bat — Windows one-shot setup for the redist project.
REM
REM Onboarding plan Task 4. Mirrors bootstrap.sh.
REM ASCII-only (CP1252 console policy / PP-34).
REM
REM Usage:
REM   bootstrap.bat [--with-python] [--with-api-key] [--skip-smoke]

setlocal enabledelayedexpansion

set "REPO_ROOT=%~dp0"
if "%REPO_ROOT:~-1%"=="\" set "REPO_ROOT=%REPO_ROOT:~0,-1%"

set "WITH_PYTHON=0"
set "WITH_API_KEY=0"
set "SKIP_SMOKE=0"

:parse_args
if "%~1"=="" goto args_done
if /i "%~1"=="--with-python"  (set "WITH_PYTHON=1"  & shift & goto parse_args)
if /i "%~1"=="--with-api-key" (set "WITH_API_KEY=1" & shift & goto parse_args)
if /i "%~1"=="--skip-smoke"   (set "SKIP_SMOKE=1"   & shift & goto parse_args)
if /i "%~1"=="--help" goto :show_help
if /i "%~1"=="-h"     goto :show_help
echo [FAIL] Unknown flag: %~1 ^(try --help^)
exit /b 1
:args_done

cd /d "%REPO_ROOT%"

REM ── Step 1: rustup ────────────────────────────────────────────────────────
echo.
echo [step 1] Checking rustup...
where rustup >NUL 2>&1
if errorlevel 1 (
    echo [FAIL] rustup not found. Install from https://rustup.rs and re-run this script.
    echo        ^(Windows installer: https://win.rustup.rs/x86_64^)
    exit /b 1
)
echo [OK] rustup present

REM ── Step 2: pinned toolchain ──────────────────────────────────────────────
echo.
echo [step 2] Installing pinned toolchain...
pushd redist
rustup show >NUL
popd
echo [OK] toolchain ready

REM ── Step 3: build redist ──────────────────────────────────────────────────
echo.
echo [step 3] Building redist ^(release, --locked^)...
pushd redist
cargo build --release --locked --bin redist
if errorlevel 1 (popd & echo [FAIL] cargo build failed & exit /b 1)
popd

REM ── Step 4: PATH preflight (PP-18) ────────────────────────────────────────
echo.
echo [step 4] PATH preflight...
set "EXPECTED_BIN=%REPO_ROOT%\redist\target\release\redist.exe"
if not exist "%EXPECTED_BIN%" (
    echo [FAIL] build succeeded but binary not at expected path: %EXPECTED_BIN%
    exit /b 1
)
echo [OK] binary at %EXPECTED_BIN%

REM ── Step 5: PATH update (current shell only) ──────────────────────────────
echo.
echo [step 5] PATH update...
set "PATH=%REPO_ROOT%\redist\target\release;%PATH%"
where redist >NUL 2>&1
if errorlevel 1 (echo [FAIL] redist still not on PATH after update & exit /b 1)
echo [OK] redist on PATH for this shell session
echo.
echo     To make this permanent ^(System Properties -^> Environment Variables^):
echo       Add %REPO_ROOT%\redist\target\release to your User PATH

REM ── Step 6 (optional): Python wheel via maturin ───────────────────────────
if "%WITH_PYTHON%"=="1" (
    echo.
    echo [step 6] Building redist_py PyO3 wheel via maturin...
    where py >NUL 2>&1
    if errorlevel 1 (echo [FAIL] python ^(py launcher^) required for --with-python & exit /b 1)
    where maturin >NUL 2>&1
    if errorlevel 1 (
        echo maturin not found; installing via pip...
        py -m pip install --user --quiet maturin
        if errorlevel 1 (echo [FAIL] maturin install failed & exit /b 1)
    )
    pushd redist\python\redist_py
    maturin develop --release
    if errorlevel 1 (popd & echo [FAIL] maturin build failed & exit /b 1)
    popd
    py -c "import redist_py; print('redist_py:', redist_py.__version__)"
    if errorlevel 1 (echo [FAIL] redist_py import failed after build & exit /b 1)
    echo [OK] redist_py importable
)

REM ── Step 7 (optional): API key with round-trip validation (PP-19) ─────────
if "%WITH_API_KEY%"=="1" (
    echo.
    echo [step 7] Dataverse API key setup...
    set "CRED_DIR=%APPDATA%\redist"
    set "CRED_FILE=!CRED_DIR!\credentials.toml"
    if not exist "!CRED_DIR!" mkdir "!CRED_DIR!"
    set /p "DATAVERSE_API_KEY=Enter your Harvard Dataverse API key (or press Enter to skip): "
    if "!DATAVERSE_API_KEY!"=="" (
        echo [SKIP] no API key entered
    ) else (
        echo Validating API key with one Dataverse round-trip...
        REM Use curl ^(Windows 10+ ships curl.exe^)
        for /f %%i in ('curl -s -o NUL -w "%%{http_code}" -H "X-Dataverse-key: !DATAVERSE_API_KEY!" "https://dataverse.harvard.edu/api/users/:me"') do set "HTTP=%%i"
        if not "!HTTP!"=="200" (
            echo [FAIL] API key validation failed ^(HTTP !HTTP!^); not writing to !CRED_FILE!
            exit /b 1
        )
        REM Atomic-ish write
        set "TMP_CRED=!CRED_DIR!\credentials.tmp"
        ^(echo [dataverse]^)>"!TMP_CRED!"
        ^(echo api_key = "!DATAVERSE_API_KEY!"^)>>"!TMP_CRED!"
        move /Y "!TMP_CRED!" "!CRED_FILE!" >NUL
        echo [OK] API key validated and written to !CRED_FILE!
    )
)

REM ── Step 8: smoke test (real run, not --print-only) PP-20 ─────────────────
if "%SKIP_SMOKE%"=="1" (
    echo.
    echo [step 8] Smoke test SKIPPED ^(--skip-smoke^)
    goto :final
)
echo.
echo [step 8] Smoke test ^(real run^)...
set "SMOKE_DIR=%TEMP%\redist-bootstrap-smoke-%RANDOM%"
mkdir "%SMOKE_DIR%"
redist state --state VT --year 2020 --label bootstrap_test --output-base "%SMOKE_DIR%" --version v1
if errorlevel 1 (
    rmdir /S /Q "%SMOKE_DIR%" 2>NUL
    echo [FAIL] smoke test bisection failed
    exit /b 1
)
set "ASSIGN=%SMOKE_DIR%\v1\2020\plans\bootstrap_test\final_assignments.json"
if not exist "%ASSIGN%" (
    rmdir /S /Q "%SMOKE_DIR%" 2>NUL
    echo [FAIL] smoke test produced no final_assignments.json
    exit /b 1
)
where py >NUL 2>&1
if not errorlevel 1 (
    for /f %%c in ('py -c "import json,sys;print(len(json.load(open(sys.argv[1]))))" "%ASSIGN%"') do set "TRACT_COUNT=%%c"
    echo [OK] smoke test produced !TRACT_COUNT! tract assignments
) else (
    echo [OK] smoke test produced %ASSIGN% ^(python not available; tract count not verified^)
)
rmdir /S /Q "%SMOKE_DIR%" 2>NUL

:final
echo.
echo ==================================================
echo Bootstrap complete.
echo.
echo Try:
echo   redist state --state VT --year 2020
echo   redist doctor --state VT --year 2020
echo   examples\vermont-2020-walkthrough\run.bat
echo.
echo First-time docs: docs\quickstart\
echo ==================================================
endlocal
exit /b 0

:show_help
echo bootstrap.bat — Windows one-shot setup for the redist project.
echo.
echo Usage:
echo   bootstrap.bat [--with-python] [--with-api-key] [--skip-smoke]
echo.
echo See bootstrap.sh header comment for full documentation.
exit /b 0
