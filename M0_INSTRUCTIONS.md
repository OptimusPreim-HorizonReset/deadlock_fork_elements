M0 — Local verification instructions

Zweck
----
Sammle die Ausgaben von run_checks.ps1 und parse_logs.ps1, packe die relevanten Artefakte und füge sie hier in die Konversation ein, damit der Agent die verbleibenden Fehler (Build / Tests / Clippy / Headless) triagieren kann.

Voraussetzungen
---------------
- Windows PowerShell (aus Repo-Root: F:\Games\deadlock_fork_elements)
- Git-Repository aktuell
- Optional: Powershell als Administrator falls Berechtigungen nötig

Empfohlener Ablauf (aus dem Repository-Root)
-------------------------------------------
# 1) Setze deterministische Test-Threads
$env:RUST_TEST_THREADS = '1'

# 2) Stelle sicher, dass logs-Ordner existiert
New-Item -ItemType Directory -Path .\logs -ErrorAction SilentlyContinue

# 3) Führe den Prüflauf aus und schreibe stdout+stderr in eine Datei
.\run_checks.ps1 *> .\logs\run_checks_console.txt

# 4) Parst die Rohlogs und generiert logs/parsed/*.md und logs_SUMMARY.md
.\parse_logs.ps1 *> .\logs\parse_logs_console.txt

# 5) Optional: Packe die Logs zur einfachen Übertragung
Compress-Archive -Path .\logs\* -DestinationPath .\logs\logs_bundle.zip -Force

Welche Dateien bitte hier einfügen
----------------------------------
- .\logs\run_checks_console.txt (komplette Konsole von run_checks.ps1)
- .\logs\parse_logs_console.txt (komplette Konsole von parse_logs.ps1)
- build-output.txt (falls von run_checks erzeugt)
- test-output.txt
- clippy-output.txt
- headless-output.txt (falls vorhanden)
- logs\parsed\logs_parsed_compile_summary.md
- logs\parsed\logs_parsed_test_summary.md
- logs_SUMMARY.md

Fehlerbehandlung
----------------
- Falls run_checks.ps1 mit Exit-Code != 0 fehlschlägt: füge .\logs\run_checks_console.txt und die oben gelisteten partial outputs ein.
- Falls parse_logs.ps1 mit Exit-Code != 0 fehlschlägt: füge .\logs\parse_logs_console.txt ein.

Hinweis
------
Die Sandbox, in der der Assistenz-Agent läuft, kann cargo/clippy nicht zuverlässig ausführen. Daher ist ein lokaler oder CI-Lauf nötig. Sobald die Artefakte hier sind, wird der Agent die Probleme priorisieren und chirurgische Fixes vorschlagen/implementieren.

Visual verification (Organigram / Account Ledger)
-------------------------------------------------
- Stellen Sie in informations.md sicher, dass `enable_flow_dynamics_model: true` gesetzt ist.
- Starten Sie die App oder, wenn sie bereits läuft, drücken Sie im UI 'R' (Reset) oder editieren informations.md (ändert den config hash) um einen Config‑Reload auszulösen.
- Nach einem Reload/Reset main.rs führt eine einmalige (one-shot) Anwendung der deterministischen Emergence‑Regel (emergence_v1) aus, die TABLES füllt (nur wenn das Feature-Flag gesetzt ist). Öffnen Sie im Renderer das Organigram / InstitutionGraph (Cycle Render‑Mode mit 'M') und prüfen Sie, ob Einträge angezeigt werden.
- Falls keine Einträge erscheinen: stoppen Sie die laufende Simulation, rebuilden Sie neu und starten Sie dann (siehe Build/Run Schritte weiter oben).

---
Erstellt automatisch von Copilot SDK in VS Code - AI assistant.