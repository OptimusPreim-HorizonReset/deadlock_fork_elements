<#
parse_logs.ps1

Purpose:
  Organize and summarize logs produced by run_checks.ps1 into logs/originals/<timestamp>/
  and logs/parsed/*.md. Produces logs/SUMMARY.md as a quick overview.

Usage (from repository root):
  .\parse_logs.ps1
  or: .\parse_logs.ps1 -RepoRoot . -Verbose

This script is intentionally conservative: it does not modify original logs, only copies
and produces concise markdown summaries for each log type.
#>

param(
    [string]$RepoRoot = ".",
    [switch]$Verbose
)

Set-StrictMode -Version Latest
Push-Location $RepoRoot

$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$origDir = Join-Path $RepoRoot "logs\originals\$timestamp"
$parsedDir = Join-Path $RepoRoot "logs\parsed"

# create directories
New-Item -ItemType Directory -Force -Path $origDir | Out-Null
New-Item -ItemType Directory -Force -Path $parsedDir | Out-Null

$expectedFiles = @("test-output.txt","clippy-output.txt","build-output.txt","headless-output.txt")
$foundFiles = @()
foreach ($f in $expectedFiles) {
    $p = Join-Path $RepoRoot $f
    if (Test-Path $p) {
        Copy-Item $p -Destination (Join-Path $origDir $f) -Force
        $foundFiles += $f
        if ($Verbose) { Write-Host "Archived $f -> $origDir" }
    }
}

if ($foundFiles.Count -eq 0) {
    Write-Host "Keine bekannten Log‑Dateien gefunden in $RepoRoot. Erwartet: $($expectedFiles -join ', ')"
    Pop-Location
    exit 1
}

function Read-FileRaw([string]$path) {
    if (Test-Path $path) { return Get-Content $path -Raw -ErrorAction SilentlyContinue } else { return "" }
}

# ---------------------
# Parse test-output.txt
# ---------------------
$testFile = Join-Path $origDir "test-output.txt"
$testSummaryOut = Join-Path $parsedDir "test_summary.md"
if (Test-Path $testFile) {
    $text = Read-FileRaw $testFile
    $resultMatch = Select-String -InputObject $text -Pattern "test result:.*" -AllMatches
    $resultLine = ""
    if ($resultMatch.Matches.Count -gt 0) { $resultLine = $resultMatch.Matches[0].Value }

    $lines = $text -split "`r?`n"

    # collect failure blocks that start with '---- ' and go until an empty line
    $failureBlocks = @()
    for ($i = 0; $i -lt $lines.Length; $i++) {
        if ($lines[$i] -match '^----\s') {
            $sb = New-Object System.Text.StringBuilder
            $sb.AppendLine($lines[$i]) | Out-Null
            $j = $i + 1
            while ($j -lt $lines.Length -and $lines[$j] -ne '') {
                $sb.AppendLine($lines[$j]) | Out-Null
                $j++
            }
            $failureBlocks += $sb.ToString().TrimEnd()
            $i = $j
        }
    }

    # collect "panicked at" contexts
    $panicBlocks = @()
    for ($i = 0; $i -lt $lines.Length; $i++) {
        if ($lines[$i] -match 'panicked at') {
            $start = [Math]::Max(0, $i-3)
            $end = [Math]::Min($lines.Length - 1, $i + 5)
            $sb = New-Object System.Text.StringBuilder
            for ($k = $start; $k -le $end; $k++) { $sb.AppendLine($lines[$k]) | Out-Null }
            $panicBlocks += $sb.ToString().TrimEnd()
        }
    }

    # merge unique blocks
    foreach ($p in $panicBlocks) { if (-not ($failureBlocks -contains $p)) { $failureBlocks += $p } }

    # Write markdown summary
    $md = "# Test Summary - $timestamp`n`n"
    if ($resultLine -ne "") { $md += "**Result:** $resultLine`n`n" } else { $md += "**Result:** (nicht gefunden) - bitte run_checks.ps1 ausführen oder test-output.txt hochladen.`n`n" }
    $md += "## Gefundene Fehler/Panikblöcke: $($failureBlocks.Count)`n`n"
    for ($idx = 0; $idx -lt $failureBlocks.Count; $idx++) {
        $md += "### Fehler $($idx + 1)`n
```
$($failureBlocks[$idx])
```
`n"
    }
    Set-Content -Path $testSummaryOut -Value $md -Encoding UTF8
    if ($Verbose) { Write-Host "Verarbeitete Test‑Logs -> $testSummaryOut" }
}

# -------------------------
# Parse build-output.txt
# -------------------------
$buildFile = Join-Path $origDir "build-output.txt"
$buildSummaryOut = Join-Path $parsedDir "compile_summary.md"
if (Test-Path $buildFile) {
    $text = Read-FileRaw $buildFile
    $lines = $text -split "`r?`n"
    $compileErrors = @()
    for ($i = 0; $i -lt $lines.Length; $i++) {
        if ($lines[$i] -match '^\s*error(\[E[0-9]+\])?:') {
            $sb = New-Object System.Text.StringBuilder
            $start = [Math]::Max(0, $i-2)
            for ($k = $start; $k -lt [Math]::Min($lines.Length, $i + 12); $k++) {
                if ($lines[$k] -eq '') { break }
                $sb.AppendLine($lines[$k]) | Out-Null
            }
            $compileErrors += $sb.ToString().TrimEnd()
        }
    }
    $md = "# Compile / Build Summary - $timestamp`n`n"
    $md += "Gefundene Compile‑Fehlblöcke: $($compileErrors.Count)`n`n"
    for ($idx = 0; $idx -lt $compileErrors.Count; $idx++) {
        $md += "## Fehler $($idx + 1)`n
```
$($compileErrors[$idx])
```
`n"
    }
    Set-Content -Path $buildSummaryOut -Value $md -Encoding UTF8
    if ($Verbose) { Write-Host "Verarbeitete Build‑Logs -> $buildSummaryOut" }
}

# -------------------------
# Parse clippy-output.txt
# -------------------------
$clippyFile = Join-Path $origDir "clippy-output.txt"
$clippySummaryOut = Join-Path $parsedDir "clippy_summary.md"
if (Test-Path $clippyFile) {
    $text = Read-FileRaw $clippyFile
    $lines = $text -split "`r?`n"
    $issues = @()
    for ($i = 0; $i -lt $lines.Length; $i++) {
        if ($lines[$i] -match '^\s*(warning|error):') {
            $sb = New-Object System.Text.StringBuilder
            $sb.AppendLine($lines[$i]) | Out-Null
            for ($k = $i + 1; $k -le $i + 8 -and $k -lt $lines.Length; $k++) {
                if ($lines[$k] -eq '') { break }
                $sb.AppendLine($lines[$k]) | Out-Null
            }
            $issues += $sb.ToString().TrimEnd()
        }
    }
    $md = "# Clippy Summary - $timestamp`n`n"
    $md += "Gefundene Clippy‑Warnungen/Fehler: $($issues.Count)`n`n"
    for ($idx = 0; $idx -lt $issues.Count; $idx++) {
        $md += "## Issue $($idx + 1)`n
```
$($issues[$idx])
```
`n"
    }
    Set-Content -Path $clippySummaryOut -Value $md -Encoding UTF8
    if ($Verbose) { Write-Host "Verarbeitete Clippy‑Logs -> $clippySummaryOut" }
}

# -------------------------
# Parse headless-output.txt
# -------------------------
$headlessFile = Join-Path $origDir "headless-output.txt"
$headlessSummaryOut = Join-Path $parsedDir "headless_summary.md"
if (Test-Path $headlessFile) {
    $text = Read-FileRaw $headlessFile
    $lines = $text -split "`r?`n"
    $interesting = @()
    for ($i = 0; $i -lt $lines.Length; $i++) {
        if ($lines[$i] -imatch '(panic|error|failed|no movement|timeout|segfault|traceback)') {
            $start = [Math]::Max(0, $i-3)
            $end = [Math]::Min($lines.Length - 1, $i + 5)
            $sb = New-Object System.Text.StringBuilder
            for ($k = $start; $k -le $end; $k++) { $sb.AppendLine($lines[$k]) | Out-Null }
            $interesting += $sb.ToString().TrimEnd()
        }
    }
    $md = "# Headless Summary - $timestamp`n`n"
    $md += "Gefundene interessante Abschnitte: $($interesting.Count)`n`n"
    for ($idx = 0; $idx -lt $interesting.Count; $idx++) {
        $md += "## Item $($idx + 1)`n
```
$($interesting[$idx])
```
`n"
    }
    Set-Content -Path $headlessSummaryOut -Value $md -Encoding UTF8
    if ($Verbose) { Write-Host "Verarbeitete Headless‑Logs -> $headlessSummaryOut" }
}

# -------------------------
# Top-level index / SUMMARY.md
# -------------------------
$top = "# Logs parsed index - $timestamp`n`n"
$top += "Archiv: logs/originals/$timestamp`n`n"
$top += "Parsed summaries: logs/parsed/`n`n"

if (Test-Path $testSummaryOut) { $top += "- Tests: logs/parsed/$(Split-Path $testSummaryOut -Leaf)`n" }
if (Test-Path $buildSummaryOut) { $top += "- Compile: logs/parsed/$(Split-Path $buildSummaryOut -Leaf)`n" }
if (Test-Path $clippySummaryOut) { $top += "- Clippy: logs/parsed/$(Split-Path $clippySummaryOut -Leaf)`n" }
if (Test-Path $headlessSummaryOut) { $top += "- Headless: logs/parsed/$(Split-Path $headlessSummaryOut -Leaf)`n" }

Set-Content -Path (Join-Path $RepoRoot "logs\SUMMARY.md") -Value $top -Encoding UTF8

Pop-Location
Write-Host "Logs organisiert: logs/originals/$timestamp und logs/parsed. Siehe logs/SUMMARY.md."