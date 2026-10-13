Abhängigkeitsanalyse und Konsistenzprüfung — Kurzbericht

Datum: 2026-09-27T04:41:19+02:00

Übersicht
--------
Dieses Dokument fasst die Modul‑/Datenabhängigkeiten der aktuellen Codebasis zusammen, hebt mögliche Konsistenzprobleme hervor und gibt konkrete Empfehlungen und Prüf-Schritte für lokale Tests.

Wichtige Module (Quelle: src/*.rs, main.rs) und deren Abhängigkeiten
------------------------------------------------------------------
- main.rs
  - Abhängigkeiten: config, renderer, simulation, tables
  - Verhalten: lädt Konfiguration, initialisiert Renderer‑Statics, startet Simulations-Loop und Hot‑Reload (Cleared TABLES on reload/reset)

- config.rs
  - Abhängigkeiten: std::fs, env
  - Verhalten: parst informations.md in einem ```config Block; stellt Defaults bereit; viele Parameter (inkl. GPDM keys)

- renderer.rs
  - Abhängigkeiten: body, quadtree/Octree, tables::TABLES, egui/quarkstrom
  - Verhalten: nutzt globale Laufzeit-Statics (FLOWFIELD_VECTOR_SCALE, FLOWFIELD_MAX_VECTORS, ORGANIGRAM_SAMPLE_LIMIT, ORGANIGRAM_CANVAS_SIZE), teilt BODIES/QUADTREE via Mutex
  - Hinweis: die Renderer‑Statics werden aus config in main.rs gesetzt und bei Reload aktualisiert

- simulation.rs
  - Abhängigkeiten: body, quadtree (Octree), config, utils, galaxy (GalaxyState)
  - Verhalten: Kern-Simulationsschleife; erzeugt bodies und octree; stellt bodies/frames für Renderer bereit

- galaxy.rs
  - Abhängigkeiten: body, accounts
  - Verhalten: fasst Körper in GalaxyState zusammen, berechnet Account‑Summen (mass, momentum, angular)

- accounts.rs
  - Keine externen crate‑Abhängigkeiten außer ultraviolet; Grundlegende Account/Delta/FlowCapacity Struktur und conservative_transfer
  - Verhalten: konservative Übertragungsprimitive (robust gegen negative Anfragen und begrenzt durch Kapazität und Verfügbarkeit)

- tables.rs
  - Abhängigkeiten: accounts, once_cell, parking_lot
  - Verhalten: Table‑Typ + TABLES Registry (Debug/Renderer only). Transfer zwischen Tables verwendet conservative_transfer

- emergence.rs
  - Abhängigkeiten: galaxy::GalaxyState, tables::Table, accounts::FlowCapacity
  - Verhalten: emergence_v1 erzeugt Table(s) wenn Schwellwert überschritten; registriert sie in TABLES, falls enable_flow_dynamics_model gesetzt ist

- contact_parliament.rs
  - Abhängigkeiten: accounts, galaxy::GalaxyState
  - Verhalten: einfache Verhandlungsfunktion (transferiert kleine Anteile von angular momentum)

- roles.rs, history.rs, renderer/organigram, etc.
  - Lokal wirken diese Module zusammen (z. B. roles im Body; history stabilitätswert). Sie sind lose gekoppelt über GalaxyState/Body/Account Strukturen.

Kritische Punkte / Risikofaktoren
---------------------------------
1. Gemeinsame globale Zustände
   - TABLES (Lazy<Mutex<Vec<Table>>>) und Renderer‑Statics + BODIES/QUADTREE sind global; Zugriff erfolgt sowohl im Simulations‑Thread als auch in UI/Renderer.
   - Empfehlung: beibehalten (sind debug/visual registries) aber Lockspezifikation prüfen: Verwende kurze Lock‑Dauern und vermeide Deadlocks (aktuell parking_lot::Mutex verwendet).

2. Konfig‑Parsing test‑Interferenz
   - InformationsConfig::load() liest aus aktuellen Verzeichnis und binary parent. Tests, die die Datei schreiben, dürfen nicht parallel laufen.
   - Empfehlung: Tests sollten in temporären Verzeichnissen laufen (ein Test wurde angepasst, um das zu tun). CI kann RUST_TEST_THREADS=1 verwenden wenn nötig.

3. Determinismus / Nebenläufigkeit in Tests
   - Simulation und Renderer‑Integration sind nicht für deterministic headless unit tests ausgelegt. "run_headless_test" nutzt kleinere n/galaxy_count. Ist geeignet für smoke tests, nicht unbedingt für deterministische assertions.

4. API‑Stabilität für Konfig‑Keys
   - Viele neue Keys (flowfield_*, organigram_*) wurden hinzugefügt; main.rs & renderer.rs müssen beim Setzen synchron bleiben (erledigt im Code).

5. Potentiale Duplicate‑Registrierung
   - emergence_v1 ersetzt Einträge mit gleicher id in TABLES, so Dubletten werden vermieden (siehe code).

6. Test‑Parallelität / Dateisystemzugriff
   - Es wurden Tests identifiziert, die auf das Dateisystem zugreifen; sicherstellen, dass sie tempdir verwenden oder mit Serialisierung laufen.

Empfohlene lokale Prüf-Schritte (Ausführen in repo root F:\Games\deadlock_fork_elements)
--------------------------------------------------------------------------------------
1) Voller Testlauf (empfohlen):
   RUST_TEST_THREADS=1 cargo test --manifest-path "F:\Games\deadlock_fork_elements\Cargo.toml"
   - Hinweis: setze RUST_TEST_THREADS=1 um Race‑Conditions durch tests die die FS verändern zu vermeiden.

2) Linting und Statische Analyse:
   cargo clippy --manifest-path "F:\Games\deadlock_fork_elements\Cargo.toml" --all-targets -- -D warnings
   cargo fmt --all

3) Release Build (Smoke):
   cargo build --release --manifest-path "F:\Games\deadlock_fork_elements\Cargo.toml"

4) Headless Smoke Run (Light GUI‑independent check):
   cargo run -- --headless-test
   - Erwartung: Simulation steigt, einige bodies bewegen sich (run_headless_test exit code 0 if OK)

Konkrete kleine Korrekturen bereits implementiert
-------------------------------------------------
- renderer: ProgressBar API Fix (ui.add(egui::ProgressBar::new(...))).
- config.rs: parsing und Defaults für neue Keys + safer unit test (tmp dir).
- tables/emergence: TABLES replacement to avoid dupes, programmer note for debug registry.

Vorschläge für nächste Schritte (ich kann sie umsetzen)
------------------------------------------------------
- Unit‑Tests erweitern: wissenschaftliche Validations-Tests (scientific_validation.rs) in eine kleine smoke subset teilen, damit sie lokal schnell ausführbar sind.
- Integrationstest für Emergence: Erzeuge GalaxyState mit verschiedenen Massen und prüfe TABLES Einträge, wenn enable_flow_dynamics_model=true.
- Clippy‑Fixes: Alle aktuellen Clippy‑Warnungen eliminieren (einige bereits adressiert), ich kann die Codebasis auf typische Clippy Patterns prüfen und minimal korrigieren.

Wenn du möchtest, führe ich sofort weitere Patches aus (z. B. serialisiere tests, erweitere Emergence‑Tests, entferne verbliebene unnötige Imports). ansonsten bitte lokal die o.g. Testbefehle laufen lassen und die Ausgabe hier posten, dann korrigiere ich zielgerichtet.

Ende der Analyse.