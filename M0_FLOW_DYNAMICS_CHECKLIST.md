# M0 Flow‑Dynamics Checkliste

Zweck
- Validierung der Flow‑Dynamics Implementation gemäss `flow-dynamics-architecture.md` und `plan.md`.

Voraussetzungen (Repo‑Root)
- PowerShell / pwsh verfügbar
- Skript: `scripts/local_m0_run.ps1`

Schnellstart‑Befehle (einmal von Repo‑Root)
```powershell
# Logs-Ordner sicherstellen
New-Item -Path .\logs -ItemType Directory -Force
# Single-threaded Tests für die Session
$env:RUST_TEST_THREADS = '1'
# Automatisierter M0-Lauf (stoppt nur Prozesse, deren ExecutablePath exakt target\release angibt)
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts\local_m0_run.ps1
```

Akzeptanzkriterien (M0)
- [ ] Unit‑Test‑Suite: `cargo test` läuft durch (keine neuen Failures, Bestätigung: `70 passed; 0 failed; 2 ignored`).
- [ ] Headless smoke: `target\debug\barnes-hut.exe --headless-test` verifiziert Partikelbewegung (Beispielausgabe: `Particle movement verification: 42 / 42 moved`).
- [ ] Release‑Build: `cargo build --release` erzeugt lauffähige Binärdateien (keine Datei‑Locks oder fehlende Artefakte).
- [ ] parsing/config: Wichtige Flow‑Konfig‑Keys sind geladen (`enable_flow_dynamics_model`, `role_update_interval`, `flow_capacity_*`).
- [ ] Emergence pipeline: Emergence‑Heuristik ist wired oder sauber dokumentiert (keine ungenutzten Emergence‑APIs ohne Begründung).
- [ ] Logs sind archiviert: `logs_bundle.zip` im Repo‑Root oder `logs/originals/...` vorhanden.
- [ ] CI‑Kopie: die Schritte oben lassen sich in CI wiederholen (same exit codes, artifacts).

Benötigte Artefakte / Logs (zum Upload für Triagierung)
- `logs/run_checks_console.txt` (komplettes stdout/stderr von run_checks)
- `logs/parse_logs_console.txt` (komplettes stdout/stderr von parse_logs)
- `logs/parsed/logs_parsed_compile_summary.md`
- `logs/parsed/logs_parsed_test_summary.md`
- `logs/parsed/logs_parsed_headless_summary.md`
- `logs_bundle.zip` (falls erstellt)
- Optional: `build-output.txt`, `test-output.txt`, `clippy-output.txt`, `headless-output.txt`

Spezifische M0‑Checks für Flow‑Dynamics (kurz)
- [ ] `src/emergence.rs`: Funktion `emergence_v1` prüfen — wird sie tatsächlich aufgerufen? Wenn nein, bestimmen, warum (implementieren vs. entfernen).
- [ ] `src/simulation.rs`: prüfe, ob `identified_compound` benötigt wird oder entfernt werden kann.
- [ ] `src/config.rs`: sicherstellen, dass Flow‑Konfig‑Keys korrekt geparst werden (keine regressiven Änderungen durch `replace`‑Chaining).
- [ ] Headless: minimaler param‑sweep (3 Seeds) um Determinismus zu prüfen.

Nächste Schritte nach M0‑Erfolg
1. M1‑Triage und Behebung funktionaler Findings (priorisiert in `M0_FLOW_DYNAMICS_TRIAGE.md`).
2. M2‑Lints: Clippy‑Fixes oder konservative `allow`‑Annotationen nach Review.
3. M3‑Headless‑Param‑Sweeps + Performance‑Profiling.

Kontakt / Hinweise
- Architekturreferenz: `flow-dynamics-architecture.md`
- Vorgabe‑Plan: `plan.md`
- SQL‑Todos: werden im Session‑DB geführt (M1…)

(Ende Checkliste)
