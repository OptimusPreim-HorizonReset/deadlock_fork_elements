$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition

$possibleCargo = @()
$cmd = Get-Command cargo -ErrorAction SilentlyContinue
if ($cmd) { $possibleCargo += $cmd.Source }
$possibleCargo += "$env:USERPROFILE\.cargo\bin\cargo.exe"
$possibleCargo += "C:\Program Files\Rust stable MSVC\bin\cargo.exe"
$possibleCargo += "C:\Program Files\Rust\bin\cargo.exe"

$cargo = $possibleCargo | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $cargo) {
    Write-Host 'Cargo wurde nicht gefunden.' -ForegroundColor Red
    Write-Host 'Bitte starte ein neues Terminal oder installiere Rust erneut.'
    Write-Host 'Alternativ:'
    Write-Host "  & 'C:\Users\Soliak 2. PC\.cargo\bin\cargo.exe' build --release"
    exit 1
}

Set-Location $scriptDir
Write-Host "Benutze Cargo: $cargo"
& $cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host 'Build fehlgeschlagen - target\release\barnes-hut.exe wurde NICHT aktualisiert.' -ForegroundColor Red
    exit $LASTEXITCODE
}

$exe = Join-Path $scriptDir 'target\release\barnes-hut.exe'
Write-Host "Frisch gebaut: $exe"

$infoSrc = Join-Path $scriptDir 'informations.md'
$infoDst = Join-Path $scriptDir 'target\release\informations.md'
if (Test-Path $infoSrc) {
    try {
        Copy-Item -Path $infoSrc -Destination $infoDst -Force
        Write-Host "Copied informations.md -> $infoDst"
    } catch {
        Write-Host "Fehler beim Kopieren von informations.md: $_" -ForegroundColor Red
    }
} else {
    Write-Host "informations.md nicht gefunden in $scriptDir; Datei wird nicht kopiert." -ForegroundColor Yellow
}

Write-Host 'Starte diese Datei mit run-release.ps1 oder start-release.bat, um die aktuelle Version auszufuehren.'
Write-Host 'Hinweis: Alte Ordner wie target_release_* enthalten veraltete Snapshot-Builds - nicht von dort starten.'

