# Clippy Summary - 20260927-085455

Gefundene Clippy‑Warnungen/Fehler: 52

## Issue 1

`
error: function `set_counter` is never used
  --> src\id_alloc.rs:14:8
   |
14 | pub fn set_counter(v: u64) {
   |        ^^^^^^^^^^^
   |
   = note: `-D dead-code` implied by `-D warnings`
   = help: to override `-D warnings` add `#[expect(dead_code)]` or `#[allow(dead_code)]`
`

## Issue 2

`
error: struct `CompoundDefinition` is never constructed
  --> src\molecules.rs:14:12
   |
14 | pub struct CompoundDefinition {
   |            ^^^^^^^^^^^^^^^^^^
`

## Issue 3

`
error: constant `KNOWN_COMPOUNDS` is never used
  --> src\molecules.rs:20:11
   |
20 | pub const KNOWN_COMPOUNDS: &[CompoundDefinition] = &[
   |           ^^^^^^^^^^^^^^^
`

## Issue 4

`
error: function `identify_compound` is never used
   --> src\molecules.rs:153:8
    |
153 | pub fn identify_compound(composition: &[(u8, u16)]) -> Option<&'static CompoundDefinition> {
    |        ^^^^^^^^^^^^^^^^^
`

## Issue 5

`
error: function `empirical_formula` is never used
   --> src\molecules.rs:160:8
    |
160 | pub fn empirical_formula(composition: &[(u8, u16)]) -> Option<String> {
    |        ^^^^^^^^^^^^^^^^^
`

## Issue 6

`
error: function `normalize_composition` is never used
   --> src\molecules.rs:173:4
    |
173 | fn normalize_composition(composition: &[(u8, u16)]) -> Option<Vec<(u8, u16)>> {
    |    ^^^^^^^^^^^^^^^^^^^^^
`

## Issue 7

`
error: methods `recompute_institutions` and `identified_compound` are never used
   --> src\simulation.rs:164:8
    |
 43 | impl Simulation {
    | --------------- methods in this implementation
...
164 |     fn recompute_institutions(&mut self) {
    |        ^^^^^^^^^^^^^^^^^^^^^^
...
`

## Issue 8

`
error: used consecutive `str::replace` call
   --> src\config.rs:357:18
    |
357 |                   .replace(':', ".")
    |  __________________^
358 | |                 .replace(';', ".")
    | |__________________________________^ help: replace with: `replace([':', ';'], ".")`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#collapsible_str_replace
`

## Issue 9

`
error: float has excessive precision
  --> src\elements.rs:89:43
   |
89 |     element_record!(21, "Sc", "Scandium", 44.955909, 4, 3, Some(1.36)),
   |                                           ^^^^^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#excessive_precision
   = note: `-D clippy::excessive-precision` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(clippy::excessive_precision)]`
`

## Issue 10

`
error: float has excessive precision
   --> src\elements.rs:101:42
    |
101 |     element_record!(33, "As", "Arsenic", 74.921596, 4, 15, Some(2.18)),
    |                                          ^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#excessive_precision
help: consider changing the type or truncating it to
    |
`

## Issue 11

`
error: float has excessive precision
   --> src\elements.rs:109:42
    |
109 |     element_record!(41, "Nb", "Niobium", 92.906372, 5, 5, Some(1.6)),
    |                                          ^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#excessive_precision
help: consider changing the type or truncating it to
    |
`

## Issue 12

`
error: float has excessive precision
   --> src\elements.rs:113:42
    |
113 |     element_record!(45, "Rh", "Rhodium", 102.905502, 5, 9, Some(2.28)),
    |                                          ^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#excessive_precision
help: consider changing the type or truncating it to
    |
`

## Issue 13

`
error: float has excessive precision
   --> src\elements.rs:121:40
    |
121 |     element_record!(53, "I", "Iodine", 126.904473, 5, 17, Some(2.66)),
    |                                        ^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#excessive_precision
help: consider changing the type or truncating it to
    |
`

## Issue 14

`
error: float has excessive precision
   --> src\elements.rs:123:41
    |
123 |     element_record!(55, "Cs", "Cesium", 132.905452, 6, 1, Some(0.79)),
    |                                         ^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#excessive_precision
help: consider changing the type or truncating it to
    |
`

## Issue 15

`
error: float has excessive precision
   --> src\elements.rs:125:44
    |
125 |     element_record!(57, "La", "Lanthanum", 138.905477, 6, 3, Some(1.1)),
    |                                            ^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#excessive_precision
help: consider changing the type or truncating it to
    |
`

## Issue 16

`
error: float has excessive precision
   --> src\elements.rs:127:47
    |
127 |     element_record!(59, "Pr", "Praseodymium", 140.907662, 6, 3, Some(1.13)),
    |                                               ^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#excessive_precision
help: consider changing the type or truncating it to
    |
`

## Issue 17

`
error: float has excessive precision
   --> src\elements.rs:133:42
    |
133 |     element_record!(65, "Tb", "Terbium", 158.925352, 6, 3, Some(1.1)),
    |                                          ^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#excessive_precision
help: consider changing the type or truncating it to
    |
`

## Issue 18

`
error: float has excessive precision
   --> src\elements.rs:135:42
    |
135 |     element_record!(67, "Ho", "Holmium", 164.930332, 6, 3, Some(1.23)),
    |                                          ^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#excessive_precision
help: consider changing the type or truncating it to
    |
`

## Issue 19

`
error: float has excessive precision
   --> src\elements.rs:137:42
    |
137 |     element_record!(69, "Tm", "Thulium", 168.934222, 6, 3, Some(1.25)),
    |                                          ^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#excessive_precision
help: consider changing the type or truncating it to
    |
`

## Issue 20

`
error: float has excessive precision
   --> src\elements.rs:141:43
    |
141 |     element_record!(73, "Ta", "Tantalum", 180.947882, 6, 5, Some(1.5)),
    |                                           ^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#excessive_precision
help: consider changing the type or truncating it to
    |
`

## Issue 21

`
error: float has excessive precision
   --> src\elements.rs:151:42
    |
151 |     element_record!(83, "Bi", "Bismuth", 208.980401, 6, 15, Some(2.02)),
    |                                          ^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#excessive_precision
help: consider changing the type or truncating it to
    |
`

## Issue 22

`
error: float has excessive precision
   --> src\elements.rs:159:47
    |
159 |     element_record!(91, "Pa", "Protactinium", 231.035882, 7, 3, Some(1.5)),
    |                                               ^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#excessive_precision
help: consider changing the type or truncating it to
    |
`

## Issue 23

`
error: float has excessive precision
   --> src\elements.rs:160:41
    |
160 |     element_record!(92, "U", "Uranium", 238.028913, 7, 3, Some(1.38)),
    |                                         ^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#excessive_precision
help: consider changing the type or truncating it to
    |
`

## Issue 24

`
error: this `if` statement can be collapsed
   --> src\elements.rs:305:5
    |
305 | /     if first.family == ElementFamily::NobleGas || second.family == ElementFamily::NobleGas {
306 | |         if first.reactivity_score() < 0.15 || second.reactivity_score() < 0.15 {
307 | |             return BindingTendency::Inert;
308 | |         }
309 | |     }
    | |_____^
`

## Issue 25

`
error: this `if` statement can be collapsed
   --> src\renderer.rs:251:9
    |
251 | /         if !self.bodies.is_empty() {
252 | |             if self.show_bodies {
253 | |                 for body in &self.bodies {
254 | |                     if let Some((pos, z)) = self.project_point_basis(
...   |
298 | |         }
`

## Issue 26

`
error: casting to the same type is unnecessary (`f32` -> `f32`)
   --> src\renderer.rs:520:37
    |
520 |                 let radius_layout = (rect.width().min(rect.height()) * 0.4) as f32;
    |                                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: try: `(rect.width().min(rect.height()) * 0.4)`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#unnecessary_cast
    = note: `-D clippy::unnecessary-cast` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::unnecessary_cast)]`
`

## Issue 27

`
error: this `if` statement can be collapsed
   --> src\renderer.rs:567:29
    |
567 | / ...                   if dist <= node_r * 1.2 {
568 | | ...                       if best.is_none() || dist < best.unwrap().1 {
569 | | ...                           best = Some((*id, dist));
570 | | ...                       }
571 | | ...                   }
    | |_______________________^
`

## Issue 28

`
error: this function has too many arguments (8/7)
   --> src\renderer.rs:676:5
    |
676 | /     fn project_point_basis(
677 | |         &self,
678 | |         point: Vec3,
679 | |         cam_pos: Vec3,
...   |
684 | |         aspect: f32,
`

## Issue 29

`
error: this `if` statement can be collapsed
   --> src\simulation.rs:823:9
    |
823 | /         if self
824 | |             .timescale_scheduler
825 | |             .should_update(self.config.role_update_interval)
...   |
831 | |         }
    | |_________^
`

## Issue 30

`
error: this `if` statement can be collapsed
   --> src\simulation.rs:835:9
    |
835 | /         if self.frame % self.config.collision_interval == 0 {
836 | |             if renderer::COLLISIONS_ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
837 | |                 self.collide();
838 | |             }
839 | |         }
    | |_________^
`

## Issue 31

`
error: manual implementation of `.is_multiple_of()`
   --> src\simulation.rs:835:12
    |
835 |         if self.frame % self.config.collision_interval == 0 {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: replace with: `self.frame.is_multiple_of(self.config.collision_interval)`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#manual_is_multiple_of
    = note: `-D clippy::manual-is-multiple-of` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::manual_is_multiple_of)]`
`

## Issue 32

`
error: manual implementation of `.is_multiple_of()`
   --> src\simulation.rs:840:12
    |
840 |         if self.frame % self.config.attract_interval == 0 {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: replace with: `self.frame.is_multiple_of(self.config.attract_interval)`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#manual_is_multiple_of
`

## Issue 33

`
error: this `if` statement can be collapsed
   --> src\simulation.rs:849:9
    |
849 | /         if self
850 | |             .timescale_scheduler
851 | |             .should_update(self.config.institution_recompute_interval)
...   |
867 | |         }
    | |_________^
`

## Issue 34

`
error: manual implementation of `.is_multiple_of()`
   --> src\main.rs:225:8
    |
225 |     if frame % interval == 0 {
    |        ^^^^^^^^^^^^^^^^^^^^^ help: replace with: `frame.is_multiple_of(interval)`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.96.0/index.html#manual_is_multiple_of
`

## Issue 35

`
error: method `recompute_institutions` is never used
   --> src\simulation.rs:164:8
    |
 43 | impl Simulation {
    | --------------- method in this implementation
...
164 |     fn recompute_institutions(&mut self) {
    |        ^^^^^^^^^^^^^^^^^^^^^^
    |
`

## Issue 36

`
error: could not compile `barnes-hut` (bin "barnes-hut") due to 34 previous errors
warning: build failed, waiting for other jobs to finish...
error: field assignment outside of initializer for an instance created with Default::default()
    --> src\simulation.rs:1453:9
     |
1453 |         config.enable_galactic_atom_simulation = true;
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     |
note: consider initializing the variable with `config::InformationsConfig { enable_galactic_atom_simulation: true, element_atomic_numbers: vec![6, 8], element_atomic_count: 12, ..Default::default() }` and removing relevant reassignments
`

## Issue 37

`
warning: build failed, waiting for other jobs to finish...
error: field assignment outside of initializer for an instance created with Default::default()
    --> src\simulation.rs:1453:9
     |
1453 |         config.enable_galactic_atom_simulation = true;
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     |
note: consider initializing the variable with `config::InformationsConfig { enable_galactic_atom_simulation: true, element_atomic_numbers: vec![6, 8], element_atomic_count: 12, ..Default::default() }` and removing relevant reassignments
    --> src\simulation.rs:1452:9
`

## Issue 38

`
error: field assignment outside of initializer for an instance created with Default::default()
    --> src\simulation.rs:1453:9
     |
1453 |         config.enable_galactic_atom_simulation = true;
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     |
note: consider initializing the variable with `config::InformationsConfig { enable_galactic_atom_simulation: true, element_atomic_numbers: vec![6, 8], element_atomic_count: 12, ..Default::default() }` and removing relevant reassignments
    --> src\simulation.rs:1452:9
     |
`

## Issue 39

`
error: field assignment outside of initializer for an instance created with Default::default()
    --> src\simulation.rs:1478:9
     |
1478 |         config.n = 8;
     |         ^^^^^^^^^^^^^
     |
note: consider initializing the variable with `config::InformationsConfig { n: 8, enable_elemental_galaxy_pair_mode: false, ..Default::default() }` and removing relevant reassignments
    --> src\simulation.rs:1477:9
     |
`

## Issue 40

`
error: field assignment outside of initializer for an instance created with Default::default()
    --> src\simulation.rs:1493:9
     |
1493 |         config.n = 8;
     |         ^^^^^^^^^^^^^
     |
note: consider initializing the variable with `config::InformationsConfig { n: 8, enable_molecular_bonding: true, molecule_max_bodies: 4, ..Default::default() }` and removing relevant reassignments
    --> src\simulation.rs:1492:9
     |
`

## Issue 41

`
error: field assignment outside of initializer for an instance created with Default::default()
    --> src\simulation.rs:1519:9
     |
1519 |         config.n = 8;
     |         ^^^^^^^^^^^^^
     |
note: consider initializing the variable with `config::InformationsConfig { n: 8, enable_molecular_bonding: true, molecule_max_bodies: 4, ..Default::default() }` and removing relevant reassignments
    --> src\simulation.rs:1518:9
     |
`

## Issue 42

`
error: field assignment outside of initializer for an instance created with Default::default()
    --> src\simulation.rs:1554:9
     |
1554 |         config.n = 8;
     |         ^^^^^^^^^^^^^
     |
note: consider initializing the variable with `config::InformationsConfig { n: 8, molecule_identify_compounds: true, ..Default::default() }` and removing relevant reassignments
    --> src\simulation.rs:1553:9
     |
`

## Issue 43

`
error: field assignment outside of initializer for an instance created with Default::default()
    --> src\simulation.rs:1568:9
     |
1568 |         config.n = 8;
     |         ^^^^^^^^^^^^^
     |
note: consider initializing the variable with `config::InformationsConfig { n: 8, ..Default::default() }` and removing relevant reassignments
    --> src\simulation.rs:1567:9
     |
`

## Issue 44

`
error: field assignment outside of initializer for an instance created with Default::default()
    --> src\simulation.rs:1589:9
     |
1589 |         config.n = 8;
     |         ^^^^^^^^^^^^^
     |
note: consider initializing the variable with `config::InformationsConfig { n: 8, enable_flow_dynamics_model: true, role_update_interval: 1, ..Default::default() }` and removing relevant reassignments
    --> src\simulation.rs:1588:9
     |
`

## Issue 45

`
error: field assignment outside of initializer for an instance created with Default::default()
    --> src\simulation.rs:1620:9
     |
1620 |         config.n = 8;
     |         ^^^^^^^^^^^^^
     |
note: consider initializing the variable with `config::InformationsConfig { n: 8, enable_flow_dynamics_model: true, ..Default::default() }` and removing relevant reassignments
    --> src\simulation.rs:1619:9
     |
`

## Issue 46

`
error: field assignment outside of initializer for an instance created with Default::default()
   --> src\scientific_validation.rs:217:5
    |
217 |     config.n = 4;
    |     ^^^^^^^^^^^^^
    |
note: consider initializing the variable with `config::InformationsConfig { n: 4, collision_interval: 100_000, accretion_spawn_rate: -1.0, ..Default::default() }` and removing relevant reassignments
   --> src\scientific_validation.rs:216:5
    |
`

## Issue 47

`
error: field assignment outside of initializer for an instance created with Default::default()
   --> src\scientific_validation.rs:233:5
    |
233 |     config.n = 2;
    |     ^^^^^^^^^^^^^
    |
note: consider initializing the variable with `config::InformationsConfig { n: 2, collision_interval: 1, ..Default::default() }` and removing relevant reassignments
   --> src\scientific_validation.rs:232:5
    |
`

## Issue 48

`
error: field assignment outside of initializer for an instance created with Default::default()
   --> src\scientific_validation.rs:284:5
    |
284 |     config.n = 24;
    |     ^^^^^^^^^^^^^^
    |
note: consider initializing the variable with `config::InformationsConfig { n: 24, enable_disk_equilibrium_mode: false, inner_radius: 2.0, outer_radius: 20.0, central_mass: 100.0, ..Default::default() }` and removing relevant reassignments
   --> src\scientific_validation.rs:283:5
    |
`

## Issue 49

`
error: field assignment outside of initializer for an instance created with Default::default()
   --> src\scientific_validation.rs:310:5
    |
310 |     config.enable_galactic_atom_simulation = true;
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
note: consider initializing the variable with `config::InformationsConfig { enable_galactic_atom_simulation: true, element_atomic_numbers: vec![26], element_atomic_count: 6, enable_scientific_element_gravity: true, element_prime_beta: 0.06, ..Default::default() }` and removing relevant reassignments
   --> src\scientific_validation.rs:309:5
    |
`

## Issue 50

`
error: field assignment outside of initializer for an instance created with Default::default()
   --> src\scientific_validation.rs:374:5
    |
374 |     config.n = 16;
    |     ^^^^^^^^^^^^^^
    |
note: consider initializing the variable with `config::InformationsConfig { n: 16, theta: 0.0, epsilon: 0.3, softening_scale_factor: 1.0, inflow_strength: 0.0, restore_strength: 0.0, sph_enabled: false, ..Default::default() }` and removing relevant reassignments
   --> src\scientific_validation.rs:373:5
    |
`

## Issue 51

`
error: field assignment outside of initializer for an instance created with Default::default()
   --> src\scientific_validation.rs:394:5
    |
394 |     config.n = 2;
    |     ^^^^^^^^^^^^^
    |
note: consider initializing the variable with `config::InformationsConfig { n: 2, enable_molecular_bonding: true, inflow_strength: 0.0, restore_strength: 0.0, ..Default::default() }` and removing relevant reassignments
   --> src\scientific_validation.rs:393:5
    |
`

## Issue 52

`
error: could not compile `barnes-hut` (bin "barnes-hut" test) due to 42 previous errors
`


