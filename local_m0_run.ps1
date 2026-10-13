<#
Local helper to run the M0 verification sequence for deadlock_fork_elements
Usage examples:
  powershell -NoProfile -ExecutionPolicy Bypass -File .\local_m0_run.ps1
  powershell -NoProfile -ExecutionPolicy Bypass -File .\local_m0_run.ps1 -Push -Branch "feature/flow-m0-verification"

This script runs run_checks.ps1, parses logs, compresses artifacts and optionally runs the commit_and_push helper.
#>
param(
    [switch]$Push,
    [string]$Branch = "feature/flow-m0-verification"
)

$ErrorActionPreference = 'Continue'
$repo = Get-Location
Write-Host "Starting local M0 run in: $repo"

# Ensure serial test execution for deterministic tests
$env:RUST_TEST_THREADS = "1"

# Prepare logs directory
$logDir = Join-Path $repo "logs"
if (-not (Test-Path $logDir)) {
    New-Item -ItemType Directory -Path $logDir | Out-Null
}

$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$runConsole = Join-Path $logDir "run_checks_console_$timestamp.txt"
$parseConsole = Join-Path $logDir "parse_logs_console_$timestamp.txt"

# Run the primary check runner
Write-Host "Running run_checks.ps1 (console -> $runConsole)"
try {
    & "$repo\run_checks.ps1" *>&1 | Tee-Object -FilePath $runConsole
} catch {
    Write-Warning "run_checks.ps1 reported an error: $_"
}

# Run the log parser (best-effort)
if (Test-Path "$repo\parse_logs.ps1") {
    Write-Host "Running parse_logs.ps1 (console -> $parseConsole)"
    try {
        & "$repo\parse_logs.ps1" *>&1 | Tee-Object -FilePath $parseConsole
    } catch {
        Write-Warning "parse_logs.ps1 reported an error: $_"
    }
} else {
    Write-Warning "parse_logs.ps1 not found in repo root; skipping parse step"
}

# Collect primary logs produced by run_checks into an archive for easy upload
$archiveName = "m0_logs_$timestamp.zip"
$archivePath = Join-Path $logDir $archiveName
Write-Host "Compressing logs to $archivePath"
try {
    # include all run_checks outputs and parse output if present
    $filesToZip = @()
    foreach ($f in Get-ChildItem -Path $logDir -File) { $filesToZip += $f.FullName }
    if ($filesToZip.Count -eq 0) {
        Write-Warning "No files found in $logDir to compress"
    } else {
        Compress-Archive -Path $filesToZip -DestinationPath $archivePath -Force
        Write-Host "Logs compressed: $archivePath"
    }
} catch {
    Write-Warning "Failed to compress logs: $_"
}

# Optionally run commit_and_push (requires git credentials and network)
if ($Push) {
    if (-not (Test-Path "$repo\commit_and_push.ps1")) {
        Write-Warning "commit_and_push.ps1 not present; skipping push step"
    } else {
        Write-Host "Running commit_and_push.ps1 -branch $Branch (console appended to logs)"
        $commitLog = Join-Path $logDir "commit_and_push_console_$timestamp.txt"
        try {
            & "$repo\commit_and_push.ps1" -branch $Branch *>&1 | Tee-Object -FilePath $commitLog
        } catch {
            Write-Warning "commit_and_push.ps1 reported an error: $_"
        }
    }
}

Write-Host "M0 run finished. Primary artifacts are in: $logDir"
Write-Host "Recommended next steps: upload $archivePath, open a PR if changes were committed, and paste logs/PR URL here for triage."

# Exit with success for now (the caller can inspect logs for failures)
exit 0
