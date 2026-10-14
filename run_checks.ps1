# Run all local checks for deadlock_fork_elements
# Usage: Open PowerShell in repository root (F:\Games\deadlock_fork_elements) and run:
#    .\run_checks.ps1

$ErrorActionPreference = 'Stop'
$repo = Get-Location
Write-Host "Running checks in $repo"

# Ensure serial test execution to avoid file-system races in tests that write informations.md
$env:RUST_TEST_THREADS = "1"

Write-Host "1) cargo test (running all tests)"
cargo test --manifest-path "$repo\Cargo.toml" 2>&1 | Tee-Object -FilePath test-output.txt

Write-Host "2) cargo clippy (treat warnings as errors)"
try {
    cargo clippy --manifest-path "$repo\Cargo.toml" --all-targets -- -D warnings 2>&1 | Tee-Object -FilePath clippy-output.txt
} catch {
    Write-Host "clippy failed (see clippy-output.txt)"
}

Write-Host "3) cargo fmt"
cargo fmt --all

Write-Host "4) cargo build --release"

# Attempt to stop any running barnes-hut processes to avoid locked target files on Windows
$procs = Get-Process -Name barnes-hut -ErrorAction SilentlyContinue
if ($procs) {
    Write-Host "Found running barnes-hut processes, attempting to stop them..."
    foreach ($p in $procs) {
        try {
            Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
            Write-Host "Stopped PID $($p.Id)"
        } catch {
            Write-Host "Failed to stop PID $($p.Id): $_"
        }
    }
}

# Try to remove an existing release binary if it is not locked
$releaseExe = Join-Path $repo "target\release\barnes-hut.exe"
$maxRetries = 6
$removed = $false
for ($i = 0; $i -lt $maxRetries; $i++) {
    if (Test-Path $releaseExe) {
        try {
            Remove-Item $releaseExe -Force -ErrorAction Stop
            Write-Host "Removed existing release exe."
            $removed = $true
            break
        } catch {
            Write-Host "Attempt $($i+1)/${maxRetries}: Could not remove release exe (it may be locked). Retrying in 1s..."
            Start-Sleep -Seconds 1
        }
    } else {
        $removed = $true
        break
    }
}
if (-not $removed) {
    Write-Host "Could not remove release exe after $maxRetries attempts; cargo build may still fail if the file is locked."
}

# Build release into a temporary target directory to avoid Windows file‑lock on the default target\release\barnes-hut.exe
$timestampBuild = Get-Date -Format "yyyyMMdd-HHmmss"
$tmpTargetDir = Join-Path $repo "target_release_$timestampBuild"
New-Item -ItemType Directory -Force -Path $tmpTargetDir | Out-Null
Write-Host "4) cargo build --release (tmp target: $tmpTargetDir)"
try {
    cargo build --release --manifest-path "$repo\Cargo.toml" --target-dir "$tmpTargetDir" 2>&1 | Tee-Object -FilePath build-output.txt
} catch {
    Write-Host "cargo build --release failed (see build-output.txt): $_"
}

# Note: we intentionally leave the normal target/ directory untouched to avoid issues with locked files on Windows.

Write-Host "5) headless smoke run (small)"
try {
    cargo run --manifest-path "$repo\Cargo.toml" -- --headless-test 2>&1 | Tee-Object -FilePath headless-output.txt
} catch {
    Write-Host "headless run failed or exited with non-zero code; check headless-output.txt"
}

Write-Host "All checks finished. Logs: test-output.txt, clippy-output.txt, build-output.txt, headless-output.txt"
