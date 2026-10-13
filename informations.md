# Property Overview

Diese Datei listet die wichtigsten konfigurierbaren Eigenschaften und Parameter des Projekts auf.
Das Programm lädt beim Start und bei Dateiaenderung automatisch den ersten gefundenen Konfigurationsblock aus informations.md.
Ungültige Werte werden ignoriert; für sie tritt der jeweilige Code-Default in Kraft.
Zusätzliche Ausführungsmodi können über Umgebungsvariablen aktiviert werden:

# RUN_CONFIG_EVALUATION=1 -> kurzes disk-equilibrium-Validierungsprofil ausführen und beenden.

# RUN_CONFIG_SWEEP=1 -> Toomre-Q-Sweep ausführen und beenden.

```config
# === SIMULATION BASICS ===
# dt: Zeitschrittweite pro Frame
# theta: Barnes-Hut-Winkelparameter für Octree-Approximation
# epsilon: Plummer-Softening für numerische Stabilität:
#   a = G*m*r / (r² + epsilon²)^(3/2)
# n: Anzahl der Disk-Partikel pro Galaxie (ohne Zentralmasse)

dt: 10.0
theta: 1.0
epsilon: 1.9
n: 2000


# === GALAXY STRUCTURE ===
# inner_radius: Radius der zentralen Masse / des Bulge
# outer_radius: Außenradius der galaktischen Scheibe
# central_mass: Masse des zentralen schwarzen Lochs / Galaxiezentrums
# particle_mass_range: Min/Max-Masse der generierten Element-Teilchen (Format: min, max)
#   Dieser globale Bereich wirkt als einheitlicher Baseline-Regler für alle Elemente.
#   In der Galaxy-as-Atom-Erzeugung wird er je Element spezifisch interpretiert und
#   für Bulge-, Disk- und Halo-Partikel individuell ausgesteuert.
#
# enable_elemental_mass_dimension_visualization: Wenn aktiviert, werden Partikelfarben
#   stärker von Prime-/Massendimensionen bestimmt. Wenn deaktiviert, verwendet der
#   Renderer segmenttypische Galaxienfarben, damit SMBH, Bulge, Scheibe, Gas und
#   Satelliten als typische astronomische Regionen unterscheidbar bleiben.

inner_radius: 2.0
outer_radius: 100000.0
central_mass: 100000000.10
particle_mass_range: 10.000000000066, 250.0000000001

# === SPAWN ZONE CONFIGURATION ===
# outer_ring_spawn_zone_inner_ratio: Innerer Radius der Spawn-Zone (als Verhältnis zu outer_radius)
# outer_ring_spawn_zone_outer_ratio: Äußerer Radius der Spawn-Zone (als Verhältnis zu outer_radius)
# accretion_spawn_rate: Wahrscheinlichkeit pro Schritt für einen neuen Partikel-Spawn

outer_ring_spawn_zone_inner_ratio: 0.05
outer_ring_spawn_zone_outer_ratio: 1.0
accretion_spawn_rate: 0.1

# === HYDRODYNAMIC FORCES ===
# inflow_strength: Stärke der radialen Einströmungskraft
# restore_strength: Stärke der vertikalen Rückstellungskraft

inflow_strength: 0.00000000000005
restore_strength: 0.0000000000000052

# === GAS DYNAMICS ===
# gas_enabled: Activate lightweight SPH-inspired gas particle dynamics
# gas_meta_particle_count: Target number of lightweight gas particles in galactic disks
# gas_target_neighbors: Desired neighbor count used to adapt smoothing radius
# gas_volume_enabled: Enable an additional volumetric gas component across galaxy pairs
# gas_volume_count: Number of volume gas particles distributed across the simulation region
# gas_volume_radius_factor: Radius factor used to spread volumetric gas around each galaxy pair
# gas_update_interval: Frames between full gas updates to trade detail for smoother FPS
# gas_h_min / gas_h_max: Minimum and maximum smoothing radius in world units
# gas_pressure_coefficient: Pressure strength multiplier for gas forces
# gas_condensation_strength: Strength of density-driven gas condensation toward dense regions
# gas_merge_distance_factor: Distance multiplier used for gas-gas merging checks
# gas_merge_velocity_max: Maximum relative velocity at which gas particles may merge
# gas_ignition_density: Density threshold above which dense gas may convert into a star body
# gas_ignition_temperature: Minimum temperature for gas ignition into a star body
# gas_ignition_mass: Minimum mass required for gas ignition into a star body
# gas_rest_density: Reference density for pressure calculation
# gas_cooling_rate: Rate at which gas temperature drops with density
# gas_relaxation_strength: Damping of out-of-plane gas motion to rebuild disks
# gas_render_opacity: Base opacity for gas billboards
# gas_motion_blur_strength: Motion blur effect intensity for gas rendering
# gas_density_color_scale: Strength of density-based color modulation
# gas_temperature_color_scale: Strength of temperature-based color modulation

gas_enabled: false
gas_meta_particle_count: 100000
gas_target_neighbors: 12
gas_volume_enabled: false
gas_volume_count: 180
gas_volume_radius_factor: 0.4
gas_update_interval: 2
gas_h_min: 0.3
gas_h_max: 7.0
gas_pressure_coefficient: 0.045
gas_condensation_strength: 0.12
gas_merge_distance_factor: 1.25
gas_merge_velocity_max: 0.14
gas_ignition_density: 0.0006
gas_ignition_temperature: 0.68
gas_ignition_mass: 0.00001
gas_rest_density: 0.3
gas_cooling_rate: 0.08
gas_relaxation_strength: 0.26
gas_render_opacity: 0.72
gas_motion_blur_strength: 0.38
gas_density_color_scale: 1.4
gas_temperature_color_scale: 0.9

# === GALAXY INTERACTION ===
# galaxy_separation_factor: Abstand zwischen Galaxien (multipliziert mit outer_radius)
# galaxy_count: Gesamtzahl der initial erzeugten Galaxien (paarweise angeordnet, auf gerade Zahl abgerundet).
#   Der Initialisierer erzeugt galaxy_count Galaxien als Paare (pair_count = floor(galaxy_count / 2)).
#   Paare werden in einem 3D-Volumen verteilt; die Volumengröße skaliert mit
#   galaxy_volume_scatter_factor * outer_radius * cbrt(pair_count). Für jedes Paar wird ein
#   Paar-Mittelpunkt zufällig im Volumen gewählt, die beiden Galaxien werden entlang einer
#   zufälligen Richtung um den Mittelpunkt mit einem Abstand von galaxy_separation_factor * outer_radius
#   positioniert. Für jede Galaxie wird anschließend eine initiale Bulk-Geschwindigkeit berechnet,
#   die von den Massen der beiden Partner und dem gewählten Interaktionstyp (prob_merge / prob_repeated / prob_flyby)
#   abhängt — so entsteht eine massenkonsistente initiale Positionierung und Vektorisierung.
# galaxy_volume_scatter_factor: Skaliert das Volumen, in dem alle Galaxienpaare chaotisch verteilt werden
# galaxy_mass_weighted_separation: Wenn true, skaliert der Paarabstand mit der Gesamtmasse des Paares
# galaxy_separation_mass_exponent: Exponent zur Massen-Skalierung des Abstands (z.B. 0.333)
# galaxy_cluster_enabled: Gruppiert Paare in mehrere Cluster (automatisch für 50+ Galaxien aktiviert)
# galaxy_cluster_count: Anzahl der Cluster (wenn clustering aktiviert)
# galaxy_cluster_radius_factor: Skaliert das interne Dispersionsvolumen innerhalb eines Clusters
# galaxy_cluster_motion_factor: Stärke der Clusterweiten Bewegung, die auf alle Paare im Cluster angewendet wird
# galaxy_property_variance_factor: Skaliert, wie stark sich individuelle Galaxien-Eigenschaften mit zunehmender Anzahl unterscheiden
# galaxy_binary_circular_fraction: Anteil der Paare, die als annähernd kreisförmige Binärsysteme initialisiert werden
# galaxy_log_centers_on_init: Wenn true, werden die generierten Cluster- und Paarzentren beim Start geloggt
# prob_merge: Wahrscheinlichkeit für verschmelzungsnahe Begegnungen
# prob_repeated: Wahrscheinlichkeit für periodisch wiederkehrende Begegnungen
# prob_flyby: Wahrscheinlichkeit für intensive Flybys

galaxy_separation_factor: 50.0
galaxy_count: 60
galaxy_volume_scatter_factor: 4.0
galaxy_mass_weighted_separation: true
galaxy_separation_mass_exponent: 0.333
galaxy_cluster_enabled: true
galaxy_cluster_count: 4
galaxy_cluster_radius_factor: 0.25
galaxy_cluster_motion_factor: 0.02
galaxy_property_variance_factor: 10.0
galaxy_binary_circular_fraction: 0.15
galaxy_log_centers_on_init: true
# total_body_budget: Gesamt-Grenze für simulierte Körper über alle Galaxien (inkl. Zentralmassen).
#   Wenn > 0, wird die Anzahl der Disk-Partikel "n" pro Galaxie so begrenzt, dass
#   sum(total bodies) ≤ total_body_budget gilt (je Galaxie wird ein Slot für die Zentralmasse reserviert).
#   Nützlich, um bei großen galaxy_count * n Laufzeit- und Speicher-Überläufe zu verhindern.
total_body_budget: 4000000
prob_merge: 0.35
prob_repeated: 0.35
prob_flyby: 0.30

# === INTERACTION SPEED FACTORS ===
# merge_speed_factor: Skalierung der Relativgeschwindigkeit für Merge-Szenarien
# repeated_speed_factor: Skalierung der Relativgeschwindigkeit für wiederkehrende Szenarien
# flyby_speed_factor: Skalierung der Relativgeschwindigkeit für Flybys

merge_speed_factor: 0.10
repeated_speed_factor: 0.70
flyby_speed_factor: 0.90

# === INTERACTION ANGLES ===
# merge_angle: Annäherungswinkel für Merge-Szenarien
# repeated_angle: Annäherungswinkel für wiederkehrende Begegnungen

# === PERFORMANCE TUNING ===
# performance_dynamic_interval_scaling: Aktiviert eine automatische Anpassung der
#   collision/attract-Intervalle bei sehr vielen Körpern, um FPS zu verbessern.
# performance_body_count_threshold: Anzahl der Körper, ab der dynamische
#   Intervallskalierung beginnt.
# performance_max_interval_scale: Maximaler Multiplikator für die Intervalle.
# performance_render_body_limit: Maximal sichtbare Körper im Renderer. Bei größeren
#   Zahlen werden nur Stichproben gezeichnet, um die Framerate zu verbessern.
# performance_render_sample_ratio: Verhältnis der gerenderten Körper im Vergleich
#   zum berechneten Renderlimit (0.1 = 10 % der erlaubten Körper werden gezeichnet).
# performance_high_count_mode: Aktiviert zusätzliche Hochlast-Optimierungen bei sehr
#   großen Körperzahlen. Standardmäßig aus, damit die Simulation bei normalen Counts
#   maximal genau bleibt.
# performance_disable_adaptive_updates_in_high_count: Wenn High-Count-Modus aktiv
#   ist, überspringt dieser Schalter adaptive Theta/Multi-
#   pole- und adaptive Zeitschritt-Berechnungen bei sehr vielen Körpern.
# performance_octree_reserve_factor: Multiplikator für die Octree-Reserve-Kapazität.
#   Niedrigere Werte sparen Speicher & Reserve-Kosten für große Körperzahlen.
#
# Zusätzliche Laufzeitprofilerung kann über Umgebungsvariablen aktiviert werden:
#   PROFILE_RUNTIME=1          -> Durchschnittswerte über Schritte und Renderzeit
#   PROFILE_FRAMES=50          -> Anzahl der Frames je Profilzyklus
#   PROFILE_MAX_FRAMES=500     -> Simulation nach der Profilzahl beenden
#   PROFILE_STEP_COMPONENTS=1  -> Detaillierte Sub-Komponenten-Timings pro Schritt
#
# Intern werden Kollisionsgitterpuffer jetzt wiederverwendet, um die Step-Zeit
#   bei hohen Körperzahlen zu reduzieren.

performance_dynamic_interval_scaling: true
performance_body_count_threshold: 400000
performance_max_interval_scale: 2
performance_render_body_limit: 400000
performance_render_sample_ratio: 1.0
performance_high_count_mode: false
performance_disable_adaptive_updates_in_high_count: false
performance_octree_reserve_factor: 4.0

# === PARTICLE PHYSICS ===
# orbital_speed_multiplier: Multiplikator für die Angular-Speed von neuen Partikeln
# spawn_angular_speed_base: Basis-Wert für Angular-Speed bei Spawns
# spawn_angular_speed_range: Zufällige Variation der Angular-Speed bei Spawns
# spin_speed_multiplier: Globale Skalierung der Eigendrehung aller Partikel

orbital_speed_multiplier: 0.025
spawn_angular_speed_base: 0.3
spawn_angular_speed_range: 0.5
spin_speed_multiplier: 10.0

# === BODY DEFORMATION ===
# equatorial_growth_factor: Wie viel der Äquator-Radius bei Rotation wächst
# polar_flattening_factor: Wie viel der Polar-Radius bei Rotation flacht ab
# radius_scale: Skalierungsfaktor für alle Partikel-Radii

equatorial_growth_factor: 0.18
polar_flattening_factor: 0.14
radius_scale: 0.5

# === DEPTH PERCEPTION ===
# depth_scale_factor: Faktor für die Z-Position bei der Tiefenwahrnehmung
# spacetime_dilation_factor: Stärke der Gravitationswellenstrahlung für spiralisierende Annäherung der Zentren

depth_scale_factor: 0.002
spacetime_dilation_factor: 0.5

# === DISK EQUILIBRIUM MODE ===
# enable_disk_equilibrium_mode: Aktiviert einen stabilitätsorientierten Scheibenmodus mit sech²-Vertikalprofil und driftkorrigierten Umlaufgeschwindigkeiten.
#   Wenn aktiviert, werden einige legacy Solver-Einstellungen durch den Mode ersetzt,
#   u. a. `theta`, `epsilon`, `collision_interval`, `attract_interval`, dynamische Intervallskalierung
#   und High-Count-Abkürzungen.
# disk_equilibrium_toomre_q: Ziel-Toomre-Q für die Scheibendispersion.
# disk_equilibrium_scale_height_factor: Maßstabsfaktor für die sech²-Vertikalverteilung (als Anteil von outer_radius).
# disk_equilibrium_asymmetric_drift_strength: Stärke der asymmetrischen Driftkorrektur für die Umlaufgeschwindigkeit.
# disk_equilibrium_softening_ratio: Softening als Anteil von outer_radius, wenn der Modus aktiviert ist.
#   dt_min / dt_max werden im Modus aus dt abgeleitet, um lokale Zeitschritte konsistent zu halten.
# disk_equilibrium_theta: Barnes-Hut-Öffnungswinkel für den Modus.
# Kreisbahngeschwindigkeiten verwenden dasselbe Plummer-Softening wie die Laufzeitkraft.
enable_disk_equilibrium_mode: true
disk_equilibrium_toomre_q: 1.3
disk_equilibrium_scale_height_factor: 0.025
disk_equilibrium_asymmetric_drift_strength: 0.12
disk_equilibrium_softening_ratio: 0.025
disk_equilibrium_theta: 0.60

# === GALAXY-AS-ATOM SIMULATION ===
# enable_galactic_atom_simulation: true = use the galaxy-as-atom simulation instead of paired galaxy disks
# enable_elemental_galaxy_pair_mode: true = use 118 periodic-table body types for each galaxy in pair mode (requires n > 1000)
# enable_elemental_simulation: legacy alias for enable_galactic_atom_simulation
# element_atomic_numbers: comma-separated Liste der Ordnungszahlen, die als Elementarsysteme erzeugt werden sollen
#   z. B. 1,2,6,8,26; Leer = 1..element_max_atomic_number
# element_pair_atomic_numbers: comma-separated Liste der Ordnungszahlen, die im Galaxy-Pair-Elementmodus verwendet werden sollen
#   Leer = alle 1..118 Elemente
#   Das Element-LOD behaelt pro Galaxie mindestens einen Body jedes ausgewaehlten Elements.
# element_pair_core_atomic_numbers: gesonderte Elementauswahl fuer Galaxy-Center/Core-Partikel
#   Leer = Kandidaten aus element_pair_atomic_numbers; die Auswahl erfolgt unabhaengig und stabilitaetsgewichtet
# enable_chemistry_compass_coloring: nutzt die chemischen Familienfarben aus chemiekompass.md fuer alle Element-Bodies
# enable_elemental_binding_response: koppelt die Kollisionselastizitaet an die berechnete Bindungstendenz
# elemental_binding_response_strength: 0..1, Staerke dieser analogen, inelastischen Kollisionsantwort
# element_atomic_count: Gesamtzahl der simulierten Elementarsysteme; bei größeren Werten wird die Auswahl wiederholt
# element_max_atomic_number: Fallback-Maximum, wenn keine element_atomic_numbers angegeben sind
#   Die Galaxy-as-Atom-Erzeugung sorgt dafür, dass besonders leichte Elemente wie Wasserstoff
#   mindestens 400 Partikel umfassen, damit die Struktur des Kern-/Bulge-/Scheiben-/Halo-Systems
#   ausreichend detailliert sichtbar wird.
# element_universe_scatter_factor: multiplies outer_radius to distribute galaxies in space
# element segment merge rules: jede atomare Segmentgruppe (Core/Bulge/Orbital) verschmilzt nur mit dem gleichen Segmenttyp desselben Elements.
#   Core/SMBH-Zentren fusionieren zusätzlich erst nach mehr als 5 Frames dauerhafter Berührung.
# element_center_mass_unit: base mass unit for the central SMBH / nucleus scaling
# element_orbital_mass_unit: mass scale for spawned orbital particles in galaxy-as-atom systems
# element_radius_scale: base radius scaling for generated galactic structures
#   dieses Basismaß wird per Element weiter angepasst: die 7 Perioden erhalten
#   jeweils eine eigene Größen-Dimension, die mit der primzahlbasierten
#   Stabilitätsfrequenz und der positionellen Verteilung innerhalb der Periode
#   verrechnet wird.
# element_prime_alpha: weights the prime-dimension coupling for galaxy analog magnetism
# element_prime_beta: weights the prime-dimension coupling for bulge/SMBH curvature
# element_light_energy: light interaction scaling factor for emitted radiation analog
# enable_elemental_gravitation_tuning: use element mass dimension for Octree gravitation instead of raw mass
# enable_elemental_mass_dimension_visualization: allow mass-dimension coloring in the renderer
# element_initial_sorting_mode: 0 = none, 1 = atomic number, 2 = atomic weight
# element_initial_grouping_mode: 0 = none, 1 = morphology clusters, 2 = orbital shell bands
# element_group_spacing: relative spacing between grouped element clusters
# element_group_internal_velocity_scale: adjust relative velocity inside grouped element systems
# element_sample_orbit_radius_scale: adjusts the radius of spawned sample orbit systems
# element_sample_orbit_speed_factor: adjusts the internal orbital speed of spawned samples
enable_galactic_atom_simulation: false
enable_elemental_galaxy_pair_mode: true
enable_elemental_simulation: true
element_atomic_numbers: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118
element_pair_atomic_numbers: 1, 5, 118
element_pair_core_atomic_numbers: 1, 5, 118
enable_chemistry_compass_coloring: true
enable_elemental_binding_response: true
elemental_binding_response_strength: 0.35
element_atomic_count: 400000
# element_max_atomic_number: 118
#   Fallbackwert für den Fall, dass element_atomic_numbers leer bleibt.
element_universe_scatter_factor: 1.8
element_center_mass_unit: 1.0
element_orbital_mass_unit: 1.0
element_radius_scale: 5.0
element_prime_alpha: 0.09
element_prime_beta: 0.06
element_light_energy: 2.0
enable_elemental_gravitation_tuning: true
enable_elemental_mass_dimension_visualization: false
element_initial_sorting_mode: 1
element_initial_grouping_mode: 1
element_group_spacing: 0.05
element_group_internal_velocity_scale: 1.0
element_sample_orbit_radius_scale: 1.0
element_sample_orbit_speed_factor: 1.0

# === SCIENTIFIC ELEMENT GRAVITY ===
# enable_scientific_element_gravity: Wenn true, wird die Gravitationsanalogie jedes
#   Elements aus Atomgewicht, Elektronegativitaet, Radius und Reaktivitaet abgeleitet
#   statt aus der klassischen Primzahldimension. Schwere, reaktive Elemente ziehen
#   staerker an; Edelgase bleiben schwach. Wasserstoff wird auf beta normiert.
# element_gravity_*_weight: Gewichtung der vier Terme in der Gravitationsformel.
#   Standard 1.0 / 0.4 / 0.3 / 0.2. Aenderungen erfordern Neuabstimmung der
#   visuellen Skalierung.
enable_scientific_element_gravity: true
element_gravity_mass_weight: 1.0
element_gravity_electronegativity_weight: 0.4
element_gravity_radius_weight: 0.3
element_gravity_reactivity_weight: 0.2

# === MOLECULAR BONDING ===
# enable_molecular_bonding: Wenn true, bilden geeignete unterschiedliche Elemente
#   bei Ueberlappung persistente Molekuele statt nur elastisch abzuprallen.
# molecule_bond_strength_factor: Skalierung der Bindungsfestigkeit (0..5).
# molecule_break_energy_threshold: Relative Geschwindigkeitsschwelle, bei der eine
#   Bindung wieder bricht (0..10).
# molecule_max_bodies: Maximale Anzahl an Koerpern in einem dynamischen Molekuel.
# molecule_identify_compounds: Wenn true, werden bekannte natuerliche Verbindungen
#   (H2O, NaCl, CO2, CH4, ...) anhand der Stoechiometrie erkannt und benannt.
enable_molecular_bonding: false
molecule_bond_strength_factor: 5.0
molecule_break_energy_threshold: 8.0
molecule_max_bodies: 12
molecule_identify_compounds: true

# === INTERACTIVE ELEMENT SAMPLING ===
# Open the sample chooser with the `^` key (Grave) and select an element from 1 to 118.
# Place a new element sample by right-clicking and dragging in the viewport.
# If right-click is unavailable, press `P` while the selector is open to place the sample at the cursor.
# Normal mode: left-click or left-drag places one body on the camera-facing target plane; drag distance controls its mass.
# Middle-mouse drag orbits the camera. The mouse wheel zooms even when the HUD is visible.
# Pointer actions over an open settings window remain reserved for the UI.

# === ADAPTIVE OPTIMIZATION ===
# enable_adaptive_theta: Region-basierte per-particle Barnes-Hut-Winkel
# enable_adaptive_multipole: Adaptive Multipolordnung (1=Monopol, 2=Dipol-Hook, 3=Quadrupol-Hook)
# enable_mixed_precision: Vereinfachte Distanz-Quantisierung für ferne Zellen
# enable_adaptive_timestep: Per-particle Δt nach Hermite-artigem Kriterium
# adaptive_theta_*: θᵢ für ruhige / mittlere / kritische Regionen
# adaptive_multipole_*: Lᵢ für ruhige / mittlere / kritische Regionen
# adaptive_dynamics_*_threshold: Grenzwerte für Regionsklassifikation (0..1)
# eta_timestep: η-Faktor für Δtᵢ = η·sqrt(|a|/|j|)
# dt_min / dt_max: Clamp für lokale Zeitschritte (0.0 => automatisch aus dt abgeleitet)
# max_energy_error: Feedback-Schwelle für aggressivere Approximationen
enable_adaptive_theta: true
enable_adaptive_multipole: true
enable_mixed_precision: true
enable_adaptive_timestep: true
adaptive_theta_quiet: 1.0
adaptive_theta_medium: 0.5
adaptive_theta_critical: 0.1
adaptive_multipole_quiet: 1
adaptive_multipole_medium: 2
adaptive_multipole_critical: 3
adaptive_dynamics_quiet_threshold: 0.001
adaptive_dynamics_critical_threshold: 0.35
eta_timestep: 1000.0
dt_min: 10.0
dt_max: 1000.0
max_energy_error: 0.5

# === FLOW-DYNAMICS / GPDM ADDITIONS ===
# Enable the new flow-dynamics model (migration mode). When true, the
# Simulation will activate table/institution flows, emergence heuristics and
# the timescale scheduler defined in the GPDM plan. Use this to validate the
# new model behind a config toggle before removing legacy routes.
enable_flow_dynamics_model: true

# Deterministic emergence rule selection for institutions. Use 'v1' for the
# deterministic heuristic implemented in src/emergence.rs. Future values may
# include 'v2-ml', 'experimental', etc.
institution_emergence_mode: v1

# Flow capacity and timing overrides (can be tuned at runtime)
flow_capacity_mass_per_dt: 1e9
flow_capacity_energy_per_dt: 1e9
flow_capacity_momentum_per_dt: 1e9
flow_capacity_angular_momentum_per_dt: 1e9
role_update_interval: 1000
institution_recompute_interval: 500

# Timescale overrides matching TimescaleTier defaults (can be tuned per run)
timescale_smbh: 1
timescale_star_formation: 100
timescale_galaxy: 10000
timescale_cluster: 1000000
```

## 1. Galaxie-Template / Spawn-Konfiguration

### `src/galaxy_templates.rs`

- `n` – Anzahl der Disc-Partikel ohne Zentralmasse.
- `inner_radius` – Radius der zentralen Masse / Bulge.
- `outer_radius` – Außenradius der galaktischen Scheibe.
- `central_mass` – Masse des zentralen Körpers (Schwarzes Loch / Galaxiezentrum).
- `particle_mass_range` – `(min, max)`-Werte für zufällig erzeugte Partikelmasse.
- `accretion_spawn_rate` – Wahrscheinlichkeit, dass ein neuer Partikel pro Schritt gespawnt wird.
- `outer_ring_spawn_zone_inner_radius` – innerer Radius der äußeren Ring-Spawn-Zone.
- `outer_ring_spawn_zone_outer_radius` – äußerer Radius der äußeren Ring-Spawn-Zone.

Direkte Änderungen:

- `src/galaxy_templates.rs`

## 2. Simulation und Zeitsteuerung

### `src/simulation.rs`

- `dt` – Zeitschrittweite der Simulation.
- `n` in `Simulation::new()` – Initiale Anzahl der Partikel in `uniform_disc()`.
- `theta` – Barnes-Hut-Winkelparameter für den Octree.
- `epsilon` – Softening-Faktor für Gravitationsberechnung.
- `spawn_accretion()` – Hier wird der automatische Spawn durchgeführt.
- `apply_hydrodynamic_inflow()` – Hier laufen die hydrodynamischen Rückführkräfte.

Wichtige Parameter in `spawn_accretion()`:

- `spawn_start_r` – Startradius der äußeren Ring-Spawn-Zone.
- `outer_r` – Außenradius des Spawn-Zonens.
- `accretion_spawn_rate` – Spawn-Wahrscheinlichkeit pro Schritt.

Wichtige Parameter zur Galaxien-Begegnung:

- `prob_merge` – Wahrscheinlichkeit für verschmelzungsnahe Begegnungen.
- `prob_repeated` – Wahrscheinlichkeit für periodisch wiederkehrende Begegnungen.
- `prob_flyby` – Wahrscheinlichkeit für intensive Flybys.
- `merge_speed_factor` – Geschwindigkeitsskalierung für Merge-Szenarien.
- `repeated_speed_factor` – Geschwindigkeitsskalierung für wiederkehrende Begegnungen.
- `flyby_speed_factor` – Geschwindigkeitsskalierung für Flybys.
- `merge_angle` – Annäherungswinkel für Merge-Szenarien.
- `repeated_angle` – Annäherungswinkel für wiederkehrende Begegnungen.
- `flyby_angle` – Annäherungswinkel für Flybys.

Wichtige Parameter in `apply_hydrodynamic_inflow()`:

- `inflow_strength` – Stärke des Lateraleinflusses von Norden/Süden.
- `restore_strength` – Stärke der vertikalen Rückführungsbeschleunigung.

Direkte Änderungen:

- `src/simulation.rs`

## 3. Partikel-Eigenschaften

### `src/body.rs`

- `pos` – Position im Raum.
- `vel` – Geschwindigkeit.
- `acc` – Beschleunigung.
- `mass` – Masse des Partikels.
- `base_radius` – Basisradius vor Formänderung durch Spin.
- `equatorial_radius` / `polar_radius` – Azimutale und polare Radien zur Darstellung.
- `rotation_axis` – Achse für Spin-Rotation.
- `angular_speed` – Rotationsgeschwindigkeit.

Direkte Änderungen:

- `src/body.rs`

## 4. Renderer- und Anzeigeoptionen

### `src/renderer.rs`

- `camera_target` / `camera_distance` / `camera_yaw` / `camera_pitch` / `camera_fov` – Kamera-Parameter.
- `camera_speed` / `camera_rotate_speed` – Kamerabewegungsgeschwindigkeit.
- `show_bodies` – Anzeigen der Partikel.
- `show_quadtree` – Anzeigen der Octree-Struktur.
- `show_spin_axes` – Anzeigen der Spin-Achsen.
- `show_mass_dimension_coloring` – Renderer-Färbung nach Massendimension.
- `show_element_debug_labels` – Anzeige zusätzlicher Elementdiagnosen.
- `enable_flow_dynamics_model` – Schaltet GPDM/Flow-Dynamics Visualisierungen und Emergenz-Regeln ein.
- `institution_emergence_mode` – Wählt die Emergenz‑Regelfamilie (z. B. "v1").
- `flowfield_vector_scale` – Skaliert die Länge der im FlowField gezeichneten Vektoren (World → Tip). Standard: 0.05.
- `flowfield_max_vectors` – Maximale Anzahl gezeichneter Flow-Vektoren (sample-basiert). Standard: 1024.
- `organigram_sample_limit` – Maximale Anzahl an Institutionen, die für die Organigram-Ansicht gesampelt werden. Standard: 200.
- `organigram_canvas_width` / `organigram_canvas_height` – Pixelgröße des Organigram-Canvas. Standard: 360 x 360.

Direkte Änderungen:

- `src/renderer.rs`

## 5. Sonstige relevante Dateien

- `src/main.rs` – Startpunkt und Rendering-Schleife.
- `src/utils.rs` – Erzeugung der initialen Disc-Verteilung.

Direkte Änderungen:

- `src/main.rs`
- `src/utils.rs`

---

## Hinweise

- Für direkte Anpassungen sind die oben genannten Dateien die zentralen Punkte.
- `informations.md` dient als Einstieg, um schnell zwischen Konfigurationsparametern zu wechseln.
- Ungültige Werte im `config`-Block werden verworfen; das Programm fällt auf Code-Default-Werte zurück.
- `dt_min` und `dt_max` werden automatisch aus `dt` abgeleitet, wenn sie `0.0` oder negativ gesetzt werden.
- Änderungen an `informations.md` werden zur Laufzeit erkannt, und die Simulation wird bei erkannter Änderung neu initialisiert.
