# Parsed Test Summary (aus run_checks_console.txt)

Datum: 2026-09-27 (abgeleitet aus der bereitgestellten Konsolenausgabe)

## Übersicht
- Insgesamt ausgeführte Tests (Anfangszeile): 72 tests running
- Testergebnis (zusammenfassend): **ok**
- Passed: **70**
- Failed: **0**
- Ignored: **2**
- Dauer (berichtete Zusammenfassung): 0.14s

## Ignorierte Tests (aus Ausgabe)
- scientific_validation::performance_four_x_particle_scaling_stays_below_twelve_x_runtime (ignored — hardware timing opt-in)
- scientific_validation::performance_optional_4096_body_timing_stays_within_calibrated_budget (ignored — hardware timing requires calibrated target-machine budget)

## Wichtige verifizierte Punkte (stichprobenhaft aus der Ausgabe)
- emergence tests: creation/registration behavior gegen Thresholds — bestanden
- scientific_validation tests: eine breite Palette physikalischer Eigentums- und Regressionsprüfungen — bestanden
- simulation tests: galactic atom mode, element spawn, molecule forces — bestanden

## Headless smoke run (Kurzcheck)
- Headless particle movement test (2 galaxies, 20 particles each): 42 bodies- Particle movement verification: 42 / 42 moved
- Beispielwerte aus Lauf:  - step 1: total displacement = 8.715659  - step 20: total displacement = 174.301163
## Fazit
Die Unit‑Tests in der bereitgestellten run_checks-Ausgabe sind grün (70 passed; 0 failed; 2 ignored). Das ist eine gute Grundlage für M0. Weiteres Ziel: Clippy/Compile-Warnungen bereinigen, damit die komplette run_checks-Pipeline (inkl. clippy als Fehler) sauber läuft.
