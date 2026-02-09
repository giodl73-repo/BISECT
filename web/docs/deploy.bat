@echo off
REM Educational Docs Site - Deployment Script
REM Usage: deploy.bat [--prepare-assets-only]

setlocal

echo.
echo ========================================
echo Educational Redistricting Website
echo Deployment Process
echo ========================================
echo.

REM Check for prepare-assets-only flag
if "%1"=="--prepare-assets-only" (
    echo [1/1] Preparing assets...
    python ..\..\scripts\web\generate_docs_site.py
    echo.
    echo [DONE] Assets prepared. Run 'npm run build' to build the site.
    goto :end
)

REM Full deployment
echo [1/3] Preparing assets...
python ..\..\scripts\web\generate_docs_site.py
if errorlevel 1 (
    echo [ERROR] Asset preparation failed!
    exit /b 1
)

echo.
echo [2/3] Building production site...
npm run build
if errorlevel 1 (
    echo [ERROR] Build failed!
    exit /b 1
)

echo.
echo [3/3] Preview production build...
echo Run 'npm run preview' to test the production build locally.
echo.
echo [DONE] Build complete! Files are in dist/
echo.
echo Deployment options:
echo   - GitHub Pages: Push dist/ to gh-pages branch
echo   - Netlify: Drag dist/ folder to netlify.com
echo   - Vercel: Run 'vercel --prod' in this directory
echo   - Manual: Copy dist/ contents to web server
echo.

:end
endlocal
