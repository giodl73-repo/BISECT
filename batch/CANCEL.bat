@echo off
REM Emergency kill switch - stops all Python redistricting processes
echo Killing all Python processes...
taskkill /F /IM python.exe /T 2>nul
taskkill /F /IM python3.13.exe /T 2>nul
taskkill /F /IM py.exe /T 2>nul
echo Done. All Python processes terminated.
pause
