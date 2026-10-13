# Parsed Test Summary (placeholder)

Status: Keine Test‑Logs gefunden.

Dieses Platzhalterdokument ist erzeugt worden, weil in diesem Runner keine test-output.txt vorhanden war.

Was hier später stehen wird (automatisch von parse_logs.ps1):
- Test‑Ergebnis (z.B. "test result: ok. 34 passed; 0 failed; 0 ignored; ...")
- Liste gefundener fehlschlagender Tests mit Backtrace‑Abschnitten
- Panik‑Blöcke mit Kontext

Nächste Schritte (lokal):
1) Im Repo‑Root ausführen: .\run_checks.ps1
   - Datei test-output.txt wird erstellt.
2) Im Repo‑Root ausführen: .\parse_logs.ps1 -Verbose
   - Erzeugt logs/originals/<timestamp>/ und die echten Markdown‑Summaries in logs/parsed/.
3) Entweder die erzeugte logs/parsed/test_summary.md hochladen oder den Inhalt hier einfügen. Ich analysiere dann automatisch die Fehlerblöcke und erstelle Prioritäten und Patches.

Wenn du möchtest, kannst du auch jetzt hier den Inhalt deiner test-output.txt einfügen; ich organisiere ihn dann in dieselbe Struktur und liefere eine Analyse.