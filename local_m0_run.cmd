@echo off
REM Batch wrapper to run the local_m0_run.ps1 PowerShell script.
REM Usage: local_m0_run.cmd [--push] [branchName]

setlocal
set "PS=PowerShell -NoProfile -ExecutionPolicy Bypass -Command"

REM Build arguments for PowerShell script
set "ARGS=-File \"%~dp0local_m0_run.ps1\""

if "%1"=="--push" (
    set "ARGS=%ARGS% -Push"
    if not "%2"=="" (
        set "ARGS=%ARGS% -Branch \"%2\""
    )
)

%PS% %ARGS%
endlocal
