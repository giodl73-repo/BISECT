@echo off
REM Add commit metadata to enhancement
REM Usage: add_commit.bat <enhancement_id> [commit_sha]
REM Examples:
REM   add_commit.bat 48
REM   add_commit.bat 48 abc123

cd /d "%~dp0"

if "%1"=="" (
    echo Usage: add_commit.bat ^<enhancement_id^> [commit_sha]
    echo.
    echo Examples:
    echo   add_commit.bat 48              - Capture all new commits for Enhancement 48
    echo   add_commit.bat 48 abc123       - Add specific commit abc123 to Enhancement 48
    exit /b 1
)

if "%2"=="" (
    echo Capturing commits for Enhancement %1...
    python capture_commits.py %1 --verbose
) else (
    echo Adding commit %2 to Enhancement %1...
    python capture_commits.py %1 --commit %2 --verbose
)

exit /b %ERRORLEVEL%
