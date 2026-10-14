GPDM Cutover PR - Anleitung

Zweck
-----
Schlage die Änderungen für die GPDM / Flow-Dynamics Migration als Feature-Branch vor und öffne einen PR mit CI-Gates.

Empfohlene Branch-Strategie
---------------------------
Branch-Name:
  feature/gpdm-m11-m13

Lokale Befehle (Windows PowerShell)
----------------------------------
cd "f:/Games/deadlock_fork_elements"
# optional: aktuelle Änderungen stashen
git checkout -b feature/gpdm-m11-m13

git add -A
git commit -m "GPDM: Renderer overlays (FlowField, Institution HUD), Table position, organigram; M11/M12 implementation.\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"

# Falls Tests lokal laufen sollen
cargo test --manifest-path "f:/Games/deadlock_fork_elements/Cargo.toml"

# Push und PR
git push origin feature/gpdm-m11-m13
# Erstelle einen PR über GitHub UI oder gh cli
# Beispiel (falls gh installiert):
# gh pr create --title "GPDM: Flow-Dynamics renderer overlays & organigram (M11/M12)" --body-file PR_TEMPLATE.md --base main

PR-Template (PR_TEMPLATE.md)
-----------------------------
Titel: GPDM: Flow-Dynamics renderer overlays & organigram (M11/M12)

Beschreibung:
- Implementiert FlowField-Overlay (Momentum-Vektoren, sampling) und Institution Spatial-Overlay
- Fügt globalen TABLES-Registry (für Debug/Renderer) hinzu
- Registriert emergente Tables in emergence_v1 und setzt Table.position
- Fügt interaktives Organigram (radiale Layout, Auswahl) und Account Ledger HUD Integration
- Unit-tests zu accounts/tables/emergence sollten vorhanden; bitte CI ausführen

Änderte Dateien (Auswahl):
- src/tables.rs (Table.position, TABLES registry)
- src/emergence.rs (set Table.position, register in TABLES)
- src/renderer.rs (FlowField sampling, Institution overlay, Organigram painter + selection)
- ggf. weitere Dateien (siehe git diff)

CI-Gates / Checks (erforderlich vor Merge)
-----------------------------------------
1. cargo test (Unit tests) — keine fehlgeschlagenen Tests
2. scientific_validation (falls als Test verfügbar) — grün
3. Smoke run: cargo run (headless oder grafisch) — keine Panics, UI-Modi schaltbar (Taste M)
4. Optional: Performance smoke for large N

Screenshots / Artifacts
-----------------------
- Bitte Screenshots des neuen RenderModes (FlowField, InstitutionGraph, AccountLedger) anhängen
- Wenn möglich, kurzes Video (≤30s) der UI Interaktion (Organigram Auswahl)

Hinweis zur Testausführung in dieser Umgebung
---------------------------------------------
Die automatische Testausführung konnte von diesem Agenten nicht lokal durchgeführt werden (Umgebungsberechtigungen/Containerzugriff beschränkt). Bitte lokal oder in CI die oben genannten Tests ausführen und das Ergebnis im PR verlinken.

Zusätzliche Hinweise
--------------------
- TABLES ist als leichte Debug/Visualisierungs-Registry konzipiert. Bei späterer Migration zur Produktions-Instanzverwaltung entsprechend anpassen.
- Konfigurationsparameter für Vector-Scaling und Sampling-Limits können später in informations.md / InformationsConfig ergänzt werden.

