# M0 Flow‑Dynamics Priorisierte Triage

Kurzüberblick
- Basisbefund (aus `logs/run_checks_console.txt` und den geparsten Summaries): Unit‑Tests und Headless Smoke sind grün. Die Pipeline fällt aber in der Clippy‑Phase wegen mehrerer Lint‑Fehler durch.

Priorität: Hoch — Funktionalität / Integrations‑Blocker

1) Emergence API ist ungenutzt / Wiring fehlt
- **Symptom:** Clippy meldet `pub fn emergence_v1` als `never used` (`src/emergence.rs`).
- **Warum wichtig:** Architekturdokument beschreibt Emergence als Kernmechanismus zur Institutionserzeugung. Unbenutzte Emergence‑Funktion deutet auf fehlende Integration in `Simulation::step()`/scheduler.
- **Betroffene Dateien:** `src/emergence.rs`, `src/simulation.rs`, `src/tables.rs` (geplant)
- **Vorschlag:**
  - Untersuchung: Suche nach Stellen in `Simulation::step()` bzw. GalaxyState-Fabrik, wo Emergence aufgerufen werden sollte.
  - Wenn Emergence‑Hook fehlt: Implementiere einen deterministischen Emergence‑Aufruf (parametrisiert hinter `enable_flow_dynamics_model`) und fülle einfache Heuristik aus `flow-dynamics-architecture.md`.
  - Tests: Unit‑Test, dass Emergence bei stabiler Region ein `Table` anlegt (mocked octree region).
- **Blocker:** Ja — M0‑Gate (falls Emergence Teil der geforderten M0‑Akzeptanz ist). Wenn M0 akzeptiert wird ohne Emergence integriert, dokumentieren.

2) Unbenutzte Hilfsmethoden in `src/simulation.rs` (`identified_compound`)
- **Symptom:** Clippy `never used` für `identified_compound`.
- **Vorschlag:** Überprüfen, ob die Funktion veraltet ist. Falls benötigt, wieder einbinden; sonst entfernen oder mit `#[allow(dead_code)]` annotieren.
- **Blocker:** Mittel — betrifft Funktionalität nur, wenn Simulation‑Funktionen davon abhängen.

Priorität: Mittel — Parsing/Config/Robustheit

3) `src/config.rs` — `replace`‑Chaining (collapsible_str_replace)
- **Symptom:** Clippy empfiehlt `replace([':', ';'], " . ")` statt Kaskade.
- **Warum:** Kann auf Regressionsrisiko bei config parsing hinweisen (Fehler beim Trennen von Listen/keys).
- **Vorschlag:** Überarbeiten: benutze eine robustere Parse‑Routine oder `replace(&[':', ';'][..], ".")` (wie Clippy vorschlägt), füge Tests für betroffene config‑strings hinzu.
- **Blocker:** Niedrig/Medium — wenn Config‑Parsing falsch ist, Emergence/roles können falsch parametriert sein.

4) Float‑Literal Precision (`src/elements.rs`) — `excessive_precision`
- **Symptom:** Viele float literals mit übermässiger Dezimalgenauigkeit.
- **Warum:** Stil, Clippy‑Fehler; meist nicht funktional, aber failt die Clippy‑Phase.
- **Vorschlag:**
  - Option A (nachhaltig): Runden/trunkieren Literale auf repräsentative Genauigkeit (z. B. 6 Dez Stellen) und Commit.
  - Option B (schnell): Datei‑level `#![allow(clippy::excessive_precision)]` mit Kommentar "data is canonical from source X".
- **Blocker:** Niedrig (stil), wird aber Clippy‑Phase aktuell stoppen.

Priorität: Niedrig — Stil/Refactor

5) `collapsible_if`, `manual_is_multiple_of`, `upper_case_acronyms`
- **Symptom:** mehrere Clippy‑Hinweise in `src/elements.rs`, `src/renderer.rs`, `src/timescales.rs`.
- **Vorschlag:** gezielte Refactors (collapse nested ifs; use `is_multiple_of`; rename enum variant or add allow + comment). Keines dieser Probleme sollte die Logik ändern, aber beseitigen Clippy‑Fehler.

Operative Tasks / Vorschläge für PRs (konkret)

- PR A (High‑Impact, small): `feature/m0-emergence-wireup`
  - Änderungen: kleine Integration in `Simulation::step()` um `emergence_v1` aufzurufen, guard mittels `enable_flow_dynamics_model`.
  - Tests: Unit test, Headless smoke run reproduziert institution creation.
  - Ziel: Beseitigt `dead_code`‑Warnung durch Verwendung oder beweist, dass Emergence intentionally unused.

- PR B (Medium): `feature/m0-config-parse-fix`
  - Änderungen: `src/config.rs` – robustere replace/parse + unit tests for affected keys.

- PR C (Split/Group): `feature/m0-clippy-cleanup-1`
  - Änderungen: fixes for `manual_is_multiple_of`, `collapsible_if` and targeted renames / allows.
  - Rationale: Break into small commits per file for easy review.

- PR D (Triage/Temp‑Allow): `chore/m0-clippy-allow-list`
  - Änderungen: add conservative `#![allow(...)]` with explanatory comments to unblock CI quickly for M0 while PRs A–C are prepared.
  - Rationale: Short‑term unblock; mark each allow with TODO linking to intended PR that will remove it.

Zeit‑ und Risikoabschätzung (grob)
- Investigation & small wiring (Emergence call + unit test): 1–2 Arbeitstage.
- Config parsing fix + tests: 0.5–1 Tag.
- Clippy style fixes (per file): 0.25–1 Tag / Datei abhängig von Umfang.
- Float literal audit: 0.5–1 Tag (wenn nur formatting) oder länger, falls Zahlenquelle/accuracy needs verification.

Empfohlene Reihenfolge (Kurz)
1. Investigate Emergence wiring (PR A).
2. If Emergence intentionally unused, add documentation and/or remove function; otherwise implement and test.
3. Apply config parsing fix (PR B).
4. Small clippy refactors to remove logic smells (PR C).
5. If timeline tight: create PR D to add conservative allows with TODO links, then remove in follow‑ups.

Sonstiges / Hinweise
- Tests + Headless sind aktuell grün → das spricht dafür, dass viele Clippy‑Fehler kosmetisch sind, aber Emergence dead_code ist semantisch relevant.
- Bevor `allow`‑Flags eingesetzt werden: dokumentiere warum (data origin, intended removal, migration plan).

(Ende Triage)
