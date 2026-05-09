@echo off
REM Wrapper for `BISECT run` -- Rust CLI orchestrator.
REM Cutover from Python pipeline performed 2026-04-29.
REM See: docs/superpowers/plans/2026-04-29-entry-point-cutover.md
REM
REM NOTE: For best viewing, resize your terminal to at least 60 lines tall and 120 columns wide
REM This allows you to see all 50 state progress bars plus post-processing output

REM Change to project root (one level up from batch/)
cd /d "%~dp0\.."

REM Pre-flight: verify BISECT is on PATH (mitigates PP-15)
where BISECT >NUL 2>&1
if errorlevel 1 (
    echo.
    echo ERROR: 'BISECT' binary not found on PATH.
    echo Build it with: cargo build --release --manifest-path Cargo.toml
    echo Then add the resulting target/release directory to your PATH.
    echo.
    exit /b 1
)

REM Pass all arguments to the Rust binary
BISECT run %*

REM If Ctrl+C was pressed, ensure any subprocess children are terminated
if errorlevel 1 (
    echo.
    echo Run completed with non-zero exit. Check error.log under outputs/ if applicable.
)
