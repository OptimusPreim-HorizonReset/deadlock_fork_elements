Logs / Zusammenfassung (lokal organisiert)

Dieses Repository enthält Hilfs-Skripte, um lokale Build- und Test-Logs zu sammeln und in eine leicht lesbare Form zu bringen.

Wichtige Skripte:
- run_checks.ps1    - führt cargo test, cargo clippy, cargo fmt, cargo build --release und einen kurzen headless-run aus und speichert Logs im Repository-Root.
- parse_logs.ps1   - liest die von run_checks.ps1 erzeugten Logs (test-output.txt, clippy-output.txt, build-output.txt, headless-output.txt), archiviert sie nach logs/originals/<timestamp>/ und erzeugt Markdown‑Zusammenfassungen in logs/parsed/.

Empfohlener Ablauf (Windows PowerShell):
1) Im Repository‑Root ausführen:
   .\run_checks.ps1

2) Danach:
   .\parse_logs.ps1 -Verbose

3) Ergebnisse:
   - logs/originals/<timestamp>/  (Kopie der Original-Logs)
   - logs/parsed/test_summary.md
   - logs/parsed/clippy_summary.md
   - logs/parsed/compile_summary.md
   - logs/parsed/headless_summary.md
   - logs/SUMMARY.md  (kurzer Index)

Wenn du die erzeugten Logs hier posten willst, kopiere nur die relevanten Abschnitte (die Fehlerblöcke) oder hänge die Markdown‑Summaries aus logs/parsed/ an — das macht die Analyse schneller.
