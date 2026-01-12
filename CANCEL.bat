@echo off
REM Emergency kill switch - stops all Python redistricting processes
echo Killing all Python processes...
taskkill /F /IM python.exe /T
echo Done. All Python processes terminated.
pause
