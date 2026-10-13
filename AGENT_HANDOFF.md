AGENT HANDOFF — GPDM / Flow‑Dynamics Migration (M0 start)

Zweck
-----
Dieses Dokument übergibt den aktuellen Stand der GPDM / Flow‑Dynamics Migration an einen implementierenden Agenten, der direkt bei M0 beginnen kann. Es fasst den Status, die sofort auszuführenden Schritte (M0), Akzeptanzkriterien und relevante Dateien / Todos zusammen.

Wichtig: Ich bin ein AI assistant using Copilot SDK in VS Code.

Überblick — was bereits implementiert ist
-----------------------------------------
Kurze Zusammenfassung (vollständige Details in plan.md und flow-dynamics-architecture.md):
- GPDM‑Primitiven: src/accounts.rs (Account, AccountDelta, FlowCapacity, conservative_transfer).
- Institutionen / Tables: src/tables.rs (Table, TABLES registry, delegate, transfer semantics).
- Deterministische Emergence: src/emergence.rs (emergence_v1, Unit‑Tests vorhanden).
- Renderer‑Scaffolding: src/renderer.rs (FlowField Overlay, Organigram, Account Ledger). Kleine Lint‑Fix in renderer.rs wurde gemacht (unused id → _id).
- Timescale Scheduler & M4: src/timescales.rs und Integration in Simulation::step() (timescale-triggered recompute / TABLES reconciliation).
- Per‑galaxy snapshot scaffolding: src/galaxy.rs + Simulation galaxy snapshot helpers.
- ID‑Allocator: src/id_alloc.rs for stable emergent IDs.
- Verifizierungs‑Hilfen: run_checks.ps1, parse_logs.ps1, M0_INSTRUCTIONS.md, LOCAL_TESTING.md.

Ziel des implementierenden Agenten (M0)
--------------------------------------
- Führe die lokale Verifikation wie in M0_INSTRUCTIONS.md aus.
- Sammle und lade die erzeugten Logs (test, clippy, build, headless, parse-outputs).
- Falls Fehler auftreten, triagiere und erstelle präzise, chirurgische Fix‑Todos (z.B. Clippy warnings, Windows build-locks).

Akzeptanzkriterien für M0
-------------------------
M0 gilt als erledigt wenn:
1) Alle Unit‑Tests lokal laufen (cargo test) und Tests/Headless Smoke in logs dokumentiert sind.
2) cargo fmt ausgeführt wurde (kein diff).
3) cargo clippy wurde ausgeführt; verbleibende Warnungen sind dokumentiert als TODOs (Clippy wird in CI als Fehler behandelt—Ziel: keine -D warnings vor PR, aber für M0 reicht dokumentieren und priorisieren).
4) cargo build --release gelingt lokal OR es ist dokumentiert, warum (z. B. laufender Prozess sperrt target/release/barnes-hut.exe). Falls gesperrt, Workaround/Checklist bereitstellen und Build in alternativem target_dir erzeugen.
5) headless smoke run (--headless-test) läuft und kurz zusammengefasste Ergebnisse vorliegen.

Sofort auszuführende Schritte (M0)
----------------------------------
Führe diese Schritte in einer PowerShell (Repository-Root: F:\Games\deadlock_fork_elements):

1) Deterministische Test-Threads
   $env:RUST_TEST_THREADS = '1'

2) Logs‑Ordner sicherstellen
   New-Item -ItemType Directory -Path .\logs -ErrorAction SilentlyContinue

3) Voller Prüflauf (empfohlen)
   .\run_checks.ps1 *> .\logs\run_checks_console.txt
   - run_checks.ps1 führt: cargo test, cargo clippy, cargo fmt, cargo build --release (in temporäres target dir), headless smoke

4) Parse die Logs
   .\parse_logs.ps1 *> .\logs\parse_logs_console.txt

5) Artefakte sammeln
   - Hochladen oder Archivieren: Compress-Archive -Path .\logs\* -DestinationPath .\logs\logs_bundle.zip -Force

Troubleshooting: Windows "Access is denied" beim Release Build
---------------------------------------------------------------
Problem: cargo build --release kann fehlschlagen, weil target\release\barnes-hut.exe von einem laufenden Prozess gehalten wird.

Check welcher Prozess die Datei hat (Beispiel):
$exe = (Get-Item .\target\release\barnes-hut.exe -ErrorAction SilentlyContinue)
if ($exe) {
  $exePath = $exe.FullName
  Get-CimInstance Win32_Process | Where-Object { $_.ExecutablePath -eq $exePath } | Select ProcessId, Name
}

Wenn Prozesse gefunden werden und sie eindeutig zur Simulation gehören, stoppe sie (nur die explizit passenden Prozesse):
Get-CimInstance Win32_Process | Where-Object { $_.ExecutablePath -eq $exePath } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }

Alternativen / Tools:
- Verwende Sysinternals handle.exe oder Process Explorer, um das Handle zu entfernen.
- Workaround: baue in ein temporäres target‑Verzeichnis: cargo build --release --target-dir target_release_YYYYMMDD-HHMMSS
  (run_checks.ps1 versucht automatisch, in ein temporäres Verzeichnis zu bauen.)

Wichtige Dateipfade & Referenzen
--------------------------------
- Hauptplan (Handoff / Roadmap): [plan.md](F:/Games/deadlock_fork_elements/plan.md)
- M0 Anweisungen: [M0_INSTRUCTIONS.md](F:/Games/deadlock_fork_elements/M0_INSTRUCTIONS.md)
- Architektur: [flow-dynamics-architecture.md](F:/Games/deadlock_fork_elements/flow-dynamics-architecture.md)
- CHANGES seit Migration: [CHANGES_SINCE_MIGRATION.md](F:/Games/deadlock_fork_elements/CHANGES_SINCE_MIGRATION.md)
- Verifikations-Skripte: [run_checks.ps1](F:/Games/deadlock_fork_elements/run_checks.ps1), [parse_logs.ps1](F:/Games/deadlock_fork_elements/parse_logs.ps1)
- Relevante Quell-Dateien: src/accounts.rs, src/tables.rs, src/emergence.rs, src/simulation.rs, src/timescales.rs, src/id_alloc.rs, src/renderer.rs

Session-SQL TODOs (Startreihenfolge)
-----------------------------------
Die Session‑DB enthält eine priorisierte Liste. Empfohlenes Vorgehen:
1) m0-run-local-verification — Run checks & collect logs (dieser Task wurde auf 'pending' gesetzt so der Agent starten kann)
2) m1-triage-and-fix-critical-findings — Triage: fixes für failing tests, compile errors, blocking clippy errors
3) m2-complete-clippy-cleanup — Kleine chirurgische Lints (unused vars, dead_code, assert! simplifications)
4) m3-run-headless-smoke-and-tune — Headless parameter sweeps, tune thresholds
5) m4-prepare-pr-and-ci — Prepare PR summary, CI gating

SQL‑Hilfsbefehle (für automatisierte Agenten)
---------------------------------------------
- Start M0 (markieren):
  UPDATE todos SET status = 'in_progress', updated_at = CURRENT_TIMESTAMP WHERE id = 'm0-run-local-verification';

- M0 erfolgreich:
  UPDATE todos SET status = 'done', updated_at = CURRENT_TIMESTAMP WHERE id = 'm0-run-local-verification';

- Fehler/Blocker: setze status = 'blocked' und schreibe detaillierte Beschreibung in das `description` Feld.

Was zu tun ist, wenn Tests/Clippy/Build scheitern
------------------------------------------------
- Dokumentiere Fehlermeldungen in logs (test-output.txt, clippy-output.txt, build-output.txt)
- Erzeuge neue SQL‑Todos mit präziser Beschreibung (z. B. fix-clippy-unused-imports) und verlinke relevante Dateien
- Bevor Änderungen committet werden: lokale Tests + clippy + fmt + headless smoke lokal ausführen

Hinweis zur Codebasis & Übergabe
--------------------------------
- Der neue Flow‑Dynamics‑Modus ist hinter dem Feature‑Flag `enable_flow_dynamics_model` konfigurierbar (in informations.md). Standard‑Aktionen sollen rückwärtskompatibel bleiben.
- Unit‑Tests existieren für Accounts/Tables/Timescales/Emergence. Testausführung mit RUST_TEST_THREADS=1 empfohlen.
- Kleine Lint‑Fixes sind bereits teilweise gemacht (z. B. renderer id → _id). Weitere kleine Clippy‑Fixes sind als Todos vorhanden.

Kontakt / Meta
---------------
- Session‑Checkpoints sind im Copilot Session State vorhanden; die wichtigsten Dateien: plan.md, M0_INSTRUCTIONS.md, flow-dynamics-architecture.md.
- Wenn Unklarheiten auftreten, schreibe neue SQL‑Todo(s) mit präzisen Reproduktionen/Logs — der nächsten Agent kann diese sequentiell abarbeiten.

-- Ende AGENT HANDOFF --
