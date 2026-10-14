Local testing and debugging instructions

Purpose:
This file contains a concise checklist and guidance for running tests, gathering logs, and how to report results back so the implementing agent (me) can continue fixing issues.

1) Run the prepared checks script
- Open PowerShell at the repository root (F:\Games\deadlock_fork_elements).
- Execute: .\run_checks.ps1
- This produces: test-output.txt, clippy-output.txt, build-output.txt, headless-output.txt in the repo root.
- After the checks complete, run .\parse_logs.ps1 to archive and summarize the logs into logs/originals/<timestamp>/ and logs/parsed/*.md. See LOGS_README.md and logs/README.md (if present) for details.

2) If tests fail:
- Copy the failing test sections from test-output.txt (include the test name, assertion, and the backtrace/line numbers).
- If there are compile errors, copy the compiler error block from the start of test-output.txt or build-output.txt.
- Paste these outputs into the issue message here.

3) If clippy fails (-D warnings):
- Open clippy-output.txt and identify the first 5 distinct warnings treated as errors.
- Those are the highest priority fixes; often they are small (unused imports, unnecessary mut, redundant clones, unnecessary parentheses).

4) If headless run fails or reports no movement:
- Open headless-output.txt and paste the output. Common causes: simulation configuration values (n, galaxy_count), or headless timeout due to heavy simulation; try run with smaller n via modifying test run: set environment variable or pass a different config.

5) If you want me to prepare immediate fixes without running tests locally:
- Paste any small compiler errors you saw earlier. I can prepare surgical patches (one-per-issue) and add unit tests to prevent regressions.

6) Committing & PR guidance
- Use a feature branch (feature/gpdm-<short>) and include the Co-authored-by trailer: "Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>" in the commit message.
- Include test summary in the PR description and reference the test-output.txt contents.

7) If tests are green locally
- Run cargo clippy & cargo fmt, then create the branch and push.
- If you want, post the git diff or the branch name and I will draft the PR body for you.

If you paste the logs here I will analyze them and prepare focused fixes. If you prefer, grant remote runner/CI access and I will try to run the tests from there.

Profiling
---------
- Env-Flags:
	- `PROFILE_RUNTIME=1` : aktiviere periodisches Laufzeitprofiling.
	- `PROFILE_FRAMES` : Anzahl Frames pro Profilzyklus (Standard: `50`).
	- `PROFILE_MAX_FRAMES` : Beende den Prozess nach N Frames (nützlich für CI/headless runs).
	- `PROFILE_STEP_COMPONENTS=1` : schalte feingranulare Sub-Komponenten-Timings ein.
- Beispiel (PowerShell):
```
$env:PROFILE_RUNTIME='1'; $env:PROFILE_FRAMES='50'; $env:PROFILE_MAX_FRAMES='200'; $env:PROFILE_STEP_COMPONENTS='1'; cargo run --release
```
- Komfort-Skript: `scripts/run_profiler_headless.ps1` (Windows PowerShell) — Aufruf z.B.:
```
.\scripts\run_profiler_headless.ps1 -MaxFrames 200 -FramesPerCycle 50 -StepComponents
```
- Ausgabe: Periodische Profiler-Zusammenfassungen erscheinen auf STDERR und (optional) werden CSV-Dateien unter `logs/profiler-<timestamp>.csv` geschrieben. CSV-Spalten:
	- `frame_start,frame_end,avg_step_ms,avg_render_ms,component,component_avg_ms`

Wenn du möchtest, kann ich zusätzlich CSV im JSON-Format schreiben oder die Komponenten noch feiner aufsplitten (z. B. Octree-Build vs. Acceleration vs. SPH). 
