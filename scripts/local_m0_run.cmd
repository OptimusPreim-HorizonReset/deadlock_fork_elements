@echo off
REM Wrapper to run local_m0_run.ps1 from anywhere in Windows
setlocal
set SCRIPT_DIR=%~dp0
powershell -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%local_m0_run.ps1" %*
endlocal
exit /b %ERRORLEVEL%