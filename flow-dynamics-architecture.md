Flow-Dynamics Architecture (Galactic Parliamentary Dynamics Model, GPDM)

Purpose
-------
This document is the engineering handoff/specification that translates the conceptual "flow-dynamics" design (flow-dynamics.txt) into a concrete implementation plan for the repository f:/Games/deadlock_fork_elements.

Scope
-----
- Provide deterministic mappings from concept terms ("Tisch", "Konten", etc.) to Rust code entities.
- Define the modules to be created/extended and the data shapes they must expose.
- Provide configuration parameters and default choices for ambiguous design decisions so implementation work can proceed.
- Record assumptions made during implementation.

High-level concept summary
--------------------------
- The simulation becomes a hierarchy of "Tables" (Institutions) that manage flows of conservation quantities (accounts): mass (M), energy (E), momentum (P), angular momentum (L), information (I) and entropy (S).
- Particles remain the lowest-level entities but gain role-weights (Akkretor, Transporteur, Heizer, Verdichter, Wind, Jet, Freier Ausreißer, Reservoir) that determine probabilistic behaviour rather than exclusive states.
- Each Table has a constitution (hard conservation rules + soft operational rules), a FlowCapacity (per-timestep transfer limits), accounts, and delegates a compact outward state (mass center, multipoles, angular momentum, feedback state) to the outer O(n log n) tree.
- Galaxy collisions become "contact parliaments" (temporary Tables) that negotiate account transfers between institutions, rather than immediate particle‑by‑particle merges.
- Multi-timescale updates: Tables operate at different Δt tiers (SMBH, star formation, galaxy, cluster), scheduled by a global scheduler.

Terminology -> Code mapping
---------------------------
- Tisch / Institution -> Table / Institution struct (module: src/tables.rs)
- Konto (M,E,P,L,I,S) -> Account struct (module: src/accounts.rs)
- Flusskapazität -> FlowCapacity struct (module: src/accounts.rs)
- Verfassung -> Constitution struct (part of tables.rs)
- Delegierter -> Delegate struct (part of tables.rs)
- Sitzanteile / Rollen -> RoleWeights struct (module: src/roles.rs), stored in Body or parallel array
- Zeitskalen-Parlamente -> TimescaleTier enum + scheduler (module: src/timescales.rs)
- Kontaktparlament -> ContactParliament type (module: src/contact_parliament.rs)
- Kosmischer Baum -> GalaxyTree (multipole-aware Octree, module: src/galaxy_tree.rs)
- Historie H -> History struct (module: src/history.rs)
- Kompressionsgrad K -> derived metric (module: src/history.rs or analytics helpers)

Re-usable existing code
-----------------------
- src/body.rs: base Body shape (pos, vel, mass, radii, gas fields). Extend with accounts, role weights, magnetization and local angular momentum.
- src/quadtree.rs: Octree implementation. Reuse core algorithms; extend Node aggregation to include angular momentum and multipoles for the delegate tree.
- src/galaxy_templates.rs: galaxy generation (spiral/elliptical). Keep Toomre-Q / disk-equilibrium code — critical to maintain physical stability.
- src/simulation.rs: currently implements a 2-galaxy setup and many behaviours (hydro inflow, accretion spawns, center dilation, collide/resolve). Rework by encapsulating per-galaxy state into GalaxyState and orchestrate via new scheduler.
- src/config.rs: already contains galaxy_count and other parameters — will be extended and used.
- src/renderer.rs: UI/egui scaffolding and global atomics — reuse to add RenderMode and overlays.
- scientific_validation.rs: extends validation suite to new accounts / flow behaviour.

Module list (new / extended)
----------------------------
- src/accounts.rs (NEW)
  - Account struct: mass, energy, momentum: Vec3, angular_momentum: Vec3, information: f32, entropy: f32
  - FlowCapacity struct: per-timestep maxima for mass, energy, momentum, angular_momentum
  - AccountDelta type and conservative transfer(from, to, requested, capacity)
  - Unit tests for conservation and clamping behaviour

- src/roles.rs (NEW)
  - RoleWeights (8 floats, normalized)
  - Role update rules (score-based) derived from local density, temperature, local angular momentum, magnetization and radiation/binding thresholds
  - Functions that convert roles to behaviour: additional accelerations, preferential spawning, ejection probabilities

- src/tables.rs (NEW)
  - Table struct: id, accounts: Account, capacity: FlowCapacity, constitution: Constitution, children: Vec<TableId>, parent: Option<TableId>, timescale: TimescaleTier
  - Institution presets: SpiralArm, Bar, SMBH, Halo, StarCluster
  - Delegate struct: exported compact state for outer tree
  - Emergence API: heuristics for detecting stable structures in Octree regions and instantiating a Table

- src/timescales.rs (NEW)
  - TimescaleTier and scheduler utilities used by Simulation::step()

- src/galaxy.rs (NEW)
  - GalaxyState encapsulating per-galaxy bodies, octree, institutions, delegates, history, equatorial plane, local config

- src/galaxy_tree.rs (NEW)
  - Multipole-aware Octree for galaxy/delegate aggregation (mass center, multipoles, angular momentum)

- src/contact_parliament.rs (NEW)
  - ContactParliament negotiation flow (Annäherung -> Überlappung -> Ressourcenverhandlung -> Sitzverteilung Umbau)

- src/history.rs (NEW)
  - History struct capturing merges, shocks, accretion history with decay; stability & fitness metrics

- src/accounts.rs, src/roles.rs and src/tables.rs will be the foundation for M1-M3.

Config additions (suggested)
---------------------------
- enable_flow_dynamics_model: bool (feature flag, default: false during migration)
- flow_capacity_* per-account defaults
- role_update_interval: usize
- institution_recompute_interval: usize
- timescale_* values (SMBH, star_formation, galaxy, cluster)
- galaxy_critical_merge_radius_factor: f32
- stability_decay_rate, history_decay_rate: f32

Assumptions made for implementation
----------------------------------
To proceed without blocking on design clarifications, the following decisions are adopted unless you instruct otherwise:
1. Compatibility: enable_flow_dynamics_model default = false. Migration proceeds behind a feature flag so existing behaviour remains reproducible during development and CI.
2. galaxy_count: use existing config default (40) for N-Galaxies; implementations will support arbitrary galaxy_count but default remains config's value.
3. Emergence rule: deterministic heuristic (not learned). For v1 the emergence of SpiralArm/Bar/other institutions will be rule-based: e.g. region stability (consistent rotation, low radial dispersion, mass threshold) over a configurable number of frames triggers new institution creation. The emergent rule is parametric and extensible later.

Implementation notes & risk mitigation
-------------------------------------
- Disk stability (Toomre-Q, vertical sech² profile, softening choice, Barnes-Hut θ) is critical. Any newly introduced role-forces (wind/jet/accretion) must be introduced with small initial coupling constants and validated against disk-equilibrium tests (use konzept3.txt guidance).
- Performance: the new galaxy-tree + per-galaxy octrees is heavier. Reuse existing performance flags (performance_mode, ultra_performance_mode) and adaptive sampling. Implement incremental profiling and add targeted tests for O(n log n) scaling.
- Validation: extend scientific_validation.rs for new invariants: account conservation under clamped transfers, role-weight normalization, emergent institution determinism, contact parliament conservation across a merge.

Next steps (correspond to Todos M1..M9)
--------------------------------------
1. M1: Implement src/accounts.rs and conservative transfer primitives.
2. M2: Implement src/roles.rs and extend Body with role fields.
3. M3: Implement src/tables.rs (institution definitions and delegates) and emergence heuristics.
4. M4: Implement timescale scheduler and integrate into Simulation::step().
5. M5: Implement GalaxyState and GalaxyTree; refactor Simulation to use galaxies: Vec<GalaxyState>.
6. M6: Implement contact parliament negotiation instead of particle-only merges.
7. M7: History/fitness/stability metrics and cutover logic (feature-flag decision point).
8. M8: Renderer overlays for 5 visualization layers and delegate organigram.
9. M9: Validation suite, docs updates, and CI build acceptance criteria.

Where this file is
------------------
- Flow architecture file in repo root: f:/Games/deadlock_fork_elements/flow-dynamics-architecture.md

Notes for the implementer agent
------------------------------
- Make small, surgical edits. Keep backward compatibility behind enable_flow_dynamics_model until validation passes.
- Add unit tests as part of each milestone; run them locally (cargo test) and the full build (cargo build --release) before marking work as complete.
- Keep the scientific validation thresholds in scientific_validation.rs as the acceptance gate for replacing the old behaviour.

Open questions (implementer should log answers in repo issues or ask the user)
-----------------------------------------------------------------------------
1. After successful migration, remove or keep the legacy 2-galaxy path? (Default: keep as compatibility mode.)
2. Target hardware scaling for final galaxy_count? (Default: keep config default 40; scale up in performance_mode only.)
3. Should emergence rules be allowed to learn (ML) in future, or remain deterministic heuristics? (Default: deterministic and parametric for v1.)

Contact
-------
- The plan and todos are stored in the project SQL todo list. Start with Todo m1 after this document is present in the repo.


(End of architecture document)
