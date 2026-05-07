@echo off
REM Wrapper for `redist run --run-type test` -- Rust CLI test/dev mode.
REM Cutover from Python pipeline performed 2026-04-29.
REM See: docs/superpowers/plans/2026-04-29-entry-point-cutover.md

REM Change to project root (one level up from batch/)
cd /d "%~dp0\.."

REM Pre-flight: verify redist is on PATH (mitigates PP-15)
where redist >NUL 2>&1
if errorlevel 1 (
    echo.
    echo ERROR: 'redist' binary not found on PATH.
    echo Build it with: cargo build --release --manifest-path redist/Cargo.toml
    echo Then add the resulting target/release directory to your PATH.
    echo.
    exit /b 1
)

REM Pass all arguments plus --run-type test to the Rust binary
redist run --run-type test %*

if errorlevel 1 (
    echo.
    echo Test run completed with non-zero exit. Check error.log under outputs/ if applicable.
)
