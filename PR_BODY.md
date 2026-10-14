PR Title: feature(gpdm): migrate to Flow‑Dynamics GPDM primitives and Emergence v1

Summary:
This PR migrates the simulation toward the GPDM Flow‑Dynamics design described in flow-dynamics.txt. Changes are intentionally surgical and feature-flagged behind enable_flow_dynamics_model. The PR includes:

- GPDM configuration keys and parser updates (src/config.rs)
- conservative_transfer, Account, FlowCapacity primitives (src/accounts.rs)
- Table abstraction, TABLES registry, and emergence v1 (src/tables.rs, src/emergence.rs)
- Renderer runtime tunables and emergence visualization (src/renderer.rs)
- Unit tests and test helpers for config parsing, transfers, and emergence
- Dependency analysis and local test helper scripts (DEPENDENCY_ANALYSIS.md, run_checks.ps1)

Verification steps (local):
1. From repository root:
   $ .\run_checks.ps1
   This runs cargo test, cargo clippy (warnings as errors), cargo fmt, and a release build. Logs are saved into test-output.txt, clippy-output.txt, build-output.txt, headless-output.txt.

2. If tests or clippy fail, inspect the logs and paste them into the review comment. The assistant will prepare minimal patches per failing item.

Notes:
- Changes default to existing behaviour unless enable_flow_dynamics_model=true in informations.md.
- TABLES is a debug registry intended for renderer-only visualization and is cleared on reload/reset.

Commit message (suggested):
"feature(gpdm): add flow-dynamics primitives, renderer tuning, and Emergence v1\n\nAdds conservative_transfer, Table registry, runtime tuning keys, and deterministic Emergence v1 behind enable_flow_dynamics_model. Includes unit tests and local test helper scripts.\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"

Branch recommendation:
feature/gpdm-migration or feature/gpdm-m11-m13

If you want, paste the git diff here (or the log outputs), and I will prepare follow-up patch commits to fix any remaining failures reported by cargo test / clippy.
