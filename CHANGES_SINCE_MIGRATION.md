Summary of changes for GPDM Flow-Dynamics migration

This file summarizes the surgical changes made during the migration to the flow-dynamics GPDM model. Use this content for the PR description or release notes.

High-level
----------
- Add GPDM feature flags and runtime tuning keys to configuration and parsing (src/config.rs).
- Implement deterministic emergence rule (emergence v1) and register emergent Tables for visualization (src/emergence.rs, src/tables.rs).
- Introduce conservative transfer primitives and account abstractions (src/accounts.rs), with unit tests to ensure conservation and clamped transfers.
- Add Renderer runtime statics allowing flowfield and organigram tuning at runtime; main.rs updates statics on load/reload/reset (src/renderer.rs, src/main.rs).
- Introduce per-galaxy snapshots (GalaxyState) and cache a galaxies vector on Simulation to support GPDM institution emergence workflows (M5) (src/galaxy.rs, src/simulation.rs).
- Replace TABLES clear-on-recompute with an id-based reconcile (upsert + retain) to avoid visual flicker during M4 recompute (src/simulation.rs, src/emergence.rs).
- Add tests for config parsing, accounts transfers, table transfers, and emergence behavior.
- Add dependency analysis and local testing helpers (DEPENDENCY_ANALYSIS.md, run_checks.ps1, LOCAL_TESTING.md).

Files changed (high level):
- src/config.rs  (new keys: enable_flow_dynamics_model, institution_emergence_mode, flowfield_vector_scale, flowfield_max_vectors, organigram_sample_limit, organigram_canvas_{width,height}, defaults, parser support, unit test)
- src/renderer.rs (runtime statics: FLOWFIELD_VECTOR_SCALE/MAX_VECTORS, ORGANIGRAM_SAMPLE_LIMIT/CANVAS_SIZE; ProgressBar API fix)
- src/main.rs (initialize and update renderer statics on load/reload/reset; config_hash updated)
- src/accounts.rs (conservative_transfer primitive + unit tests)
- src/tables.rs, src/emergence.rs (Table struct, TABLES registry, emergence_v1 + unit tests)
- DEPENDENCY_ANALYSIS.md, LOCAL_TESTING.md, run_checks.ps1, CHANGES_SINCE_MIGRATION.md

Testing guidance
----------------
- Run tests with RUST_TEST_THREADS=1 to avoid test concurrency issues that alter the repo files.
- Use run_checks.ps1 to collect logs for tests, clippy and build.

Notes
-----
- All changes are behind config flags where possible and default to the previous behavior unless explicitly enabled in informations.md.
- The TABLES registry is intentionally debug/visual-only and is cleared on config reload/reset to avoid stale entries.
- For demo/visualization convenience, main.rs now invokes the deterministic emergence rule (emergence_v1) once after a config reload or explicit UI Reset when `enable_flow_dynamics_model` is true. This populates the TABLES registry so the Organigram / Account Ledger immediately show emergent Tables. This is intentionally a one-shot demo and will be replaced by the timescale-driven recompute (M4) in follow-up work.

Next steps
----------
1) Run the full test suite and fix any residual compiler/test failures.
2) Address clippy warnings (treated as errors with -D warnings) to keep codebase clean.
3) Expand scientific_validation.rs tests and run them in CI (they may have non-trivial runtime cost).
4) Prepare PR and run CI.
