@echo off
REM Educational Docs Site - Development Server
REM Usage: dev.bat

echo.
echo ========================================
echo Educational Redistricting Website
echo Development Server Starting...
echo ========================================
echo.

cd /d "%~dp0"
npm run dev
