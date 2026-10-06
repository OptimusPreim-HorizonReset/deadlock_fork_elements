<#
.SYNOPSIS
    Automatisiert M0-Lauf lokal: stoppt ggf. Release-EXEs, führt run_checks.ps1 und parse_logs.ps1 aus,
    protokolliert Ausgabe, erstellt logs_bundle.zip.

.DESCRIPTION
    Dieses Skript ist dafür gedacht, in der Repo-Root-Umgebung ausgeführt zu werden (es ermittelt automatisch
    das Repo-Root relativ zum Skriptpfad). Es versucht nur Prozesse zu beenden, deren "ExecutablePath"
    exakt auf die Dateien in `target\release` zeigt.

.PARAMETER DryRun
    Nur ausgeben, welche Aktionen durchgeführt würden (keine Stop/Zip-Operationen).

.PARAMETER NoZip
    Kein `logs_bundle.zip` erzeugen.

.PARAMETER SkipStop
    Keine laufenden Release-EXEs beenden.

.EXAMPLE
    # Normale Ausführung
    .\scripts\local_m0_run.ps1

    # Dry run
    .\scripts\local_m0_run.ps1 -DryRun
#>

[CmdletBinding()]
param(
    [switch]$DryRun,
    [switch]$NoZip,
    [switch]$SkipStop
)

function Write-Note($msg) { Write-Host "[local_m0_run] $msg" }

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Resolve-Path (Join-Path $scriptDir "..")
$repoRoot = $repoRoot.Path

Set-Location -Path $repoRoot

$logsDir = Join-Path $repoRoot "logs"
$parsedDir = Join-Path $logsDir "parsed"
$runChecksLog = Join-Path $logsDir "run_checks_console.txt"
$runChecksExit = Join-Path $logsDir "run_checks_exit_code.txt"
$parseLogsLog = Join-Path $logsDir "parse_logs_console.txt"
$parseExit = Join-Path $logsDir "parse_logs_exit_code.txt"
$logsBundle = Join-Path $repoRoot "logs_bundle.zip"
$releaseDir = Join-Path $repoRoot "target\release"

Write-Note "Repo root: $repoRoot"
Write-Note "Logs dir: $logsDir"

# Ensure logs directory
if (-not (Test-Path $logsDir)) {
    New-Item -Path $logsDir -ItemType Directory -Force | Out-Null
}

# Stop only processes whose ExecutablePath matches a release exe
if (-not $SkipStop) {
    if (-not (Test-Path $releaseDir)) {
        Write-Note "No target/release directory found at $releaseDir"
    } else {
        $exeFiles = Get-ChildItem -Path $releaseDir -Filter *.exe -File -ErrorAction SilentlyContinue
        if (-not $exeFiles -or $exeFiles.Count -eq 0) {
            Write-Note "No release executables found in $releaseDir"
        } else {
            foreach ($exe in $exeFiles) {
                $exeFullPath = $exe.FullName
                Write-Note "Checking processes for EXE: $exeFullPath"
                try {
                    $procs = Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
                        Where-Object { $_.ExecutablePath -and ($_.ExecutablePath -ieq $exeFullPath) }
                } catch {
                    Write-Warning "Could not query Win32_Process: $_"
                    $procs = $null
                }
                if ($procs) {
                    foreach ($p in $procs) {
                        Write-Note "Found process Id $($p.ProcessId) Name $($p.Name) holding $exeFullPath"
                        if ($DryRun) {
                            Write-Note "(dry-run) Would stop PID $($p.ProcessId)"
                        } else {
                            try {
                                Stop-Process -Id $p.ProcessId -Force -ErrorAction Stop
                                Write-Note "Stopped PID $($p.ProcessId)"
                            } catch {
                                Write-Warning "Failed to stop PID $($p.ProcessId): $_"
                            }
                        }
                    }
                } else {
                    Write-Note "No processes hold $exeFullPath"
                }
            }
        }
    }
} else {
    Write-Note "Skipping stopping release processes (SkipStop set)."
}

# Set single-threaded tests for the session
$env:RUST_TEST_THREADS = '1'
Write-Note "Set RUST_TEST_THREADS=1 for this session"

# Run run_checks.ps1 and capture output
$runChecksScript = Join-Path $repoRoot 'run_checks.ps1'
if (-not (Test-Path $runChecksScript)) {
    Write-Warning "run_checks.ps1 not found at $runChecksScript"
    $rc1 = 127
    Write-Output $rc1 | Out-File -FilePath $runChecksExit -Encoding ascii
} else {
    Push-Location $repoRoot
    try {
        Write-Note "Running run_checks.ps1 -> $runChecksLog"
        & "$runChecksScript" *>&1 | Tee-Object -FilePath $runChecksLog
        $rc1 = $LASTEXITCODE
        Write-Output $rc1 | Out-File -FilePath $runChecksExit -Encoding ascii
        Write-Note "run_checks exit code: $rc1"
    } catch {
        Write-Warning "run_checks execution error: $_"
        $rc1 = 100
        Write-Output $rc1 | Out-File -FilePath $runChecksExit -Encoding ascii
    } finally {
        Pop-Location
    }
}

# Run parse_logs.ps1 and capture output
$parseLogsScript = Join-Path $repoRoot 'parse_logs.ps1'
if (-not (Test-Path $parseLogsScript)) {
    Write-Warning "parse_logs.ps1 not found at $parseLogsScript"
    $rc2 = 127
    Write-Output $rc2 | Out-File -FilePath $parseExit -Encoding ascii
} else {
    Push-Location $repoRoot
    try {
        Write-Note "Running parse_logs.ps1 -> $parseLogsLog"
        & "$parseLogsScript" *>&1 | Tee-Object -FilePath $parseLogsLog
        $rc2 = $LASTEXITCODE
        Write-Output $rc2 | Out-File -FilePath $parseExit -Encoding ascii
        Write-Note "parse_logs exit code: $rc2"
    } catch {
        Write-Warning "parse_logs execution error: $_"
        $rc2 = 101
        Write-Output $rc2 | Out-File -FilePath $parseExit -Encoding ascii
    } finally {
        Pop-Location
    }
}

# List parsed files
if (Test-Path $parsedDir) {
    $parsed = Get-ChildItem -Path $parsedDir -Filter *.md -File -ErrorAction SilentlyContinue
    if ($parsed -and $parsed.Count -gt 0) {
        Write-Note "Parsed files in ${parsedDir}:"
        $parsed | ForEach-Object { Write-Host " - $($_.FullName)" }
    } else {
        Write-Note "No parsed markdown files found in $parsedDir"
    }
} else {
    Write-Note "Parsed directory $parsedDir does not exist"
}

# Compress logs bundle
if (-not $NoZip) {
    if ($DryRun) {
        Write-Note "(dry-run) Would create $logsBundle"
    } else {
        try {
            if (Test-Path $logsBundle) { Remove-Item $logsBundle -Force -ErrorAction SilentlyContinue }
            Compress-Archive -Path (Join-Path $logsDir "*") -DestinationPath $logsBundle -Force
            Write-Note "Created $logsBundle"
        } catch {
            Write-Warning "Failed to create logs bundle: $_"
        }
    }
} else {
    Write-Note "Skipping zip (NoZip set)"
}

# Final summary and exit code
if (-not (Get-Variable -Name rc1 -Scope Script -ErrorAction SilentlyContinue)) { $rc1 = 0 }
if (-not (Get-Variable -Name rc2 -Scope Script -ErrorAction SilentlyContinue)) { $rc2 = 0 }

if ($rc1 -ne 0) {
    Write-Note "M0 result: run_checks failed (exit $rc1). See $runChecksLog"
    exit $rc1
} elseif ($rc2 -ne 0) {
    Write-Note "M0 result: parse_logs failed (exit $rc2). See $parseLogsLog"
    exit $rc2
} else {
    Write-Note "M0 run completed successfully."
    exit 0
}
