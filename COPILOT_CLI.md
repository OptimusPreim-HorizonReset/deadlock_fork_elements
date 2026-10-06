Automationsskript für M0 — Aufruf via Copilot CLI

Dieses Repo enthält `scripts\local_m0_run.ps1` (PowerShell) und `scripts\local_m0_run.cmd` (Windows‑Wrapper).
Das Skript führt diese Schritte aus:
- optional: beendet Prozesse, deren `ExecutablePath` exakt auf Dateien in `target\release` zeigt
- setzt `RUST_TEST_THREADS=1` für die Session
- führt `run_checks.ps1` aus und protokolliert Ausgabe in `logs\run_checks_console.txt`
- führt `parse_logs.ps1` aus und protokolliert Ausgabe in `logs\parse_logs_console.txt`
- erzeugt `logs_bundle.zip` mit dem Inhalt von `logs\`

Parameter:
- `-DryRun` : keine Kill- oder Zip-Operationen, nur Ausgabe
- `-NoZip`  : kein `logs_bundle.zip` erzeugen
- `-SkipStop`: keine Release-EXEs beenden

Aufrufbeispiele (aus Repo-Root):

# Direkter PowerShell-Aufruf
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts\local_m0_run.ps1

# Windows-Batch-Wrapper (einfacher)
scripts\local_m0_run.cmd

# Mit Parametern
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts\local_m0_run.ps1 -DryRun

Copilot CLI (Beispiele)

Wenn du die GitHub Copilot CLI (`copilot`) installiert hast, kannst du das Skript so ausführen:

# Variante A: Copilot führt direkt den PowerShell-Befehl aus
copilot run -- pwsh -NoProfile -ExecutionPolicy Bypass -File scripts\local_m0_run.ps1

# Variante B: Copilot führt die Batch-Wrapper-Datei aus
copilot run -- scripts\local_m0_run.cmd

Hinweis: Die exakte Syntax für `copilot run` hängt von deiner Copilot-CLI-Version ab; die obigen Beispiele rufen im Hintergrund einen normalen Shell-Befehl auf. Falls deine Copilot-CLI ein Tasks- oder Actions‑Manifest benötigt, kann ich ein solches Manifest ergänzen.

Erwartete Artefakte nach erfolgreichem Lauf (im Repo-Root):
- `logs\run_checks_console.txt`
- `logs\parse_logs_console.txt`
- `logs\parsed\logs_parsed_compile_summary.md`
- `logs\parsed\logs_parsed_test_summary.md`
- `logs_bundle.zip`

Wenn du möchtest, committe ich die neuen Dateien auf einen Feature-Branch und erstelle einen PR. Oder soll ich zusätzlich ein `.vscode/tasks.json` anlegen, damit VS Code / Copilot-Extension das Task direkt ausführt?