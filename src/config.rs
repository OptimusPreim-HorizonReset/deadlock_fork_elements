use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct InformationsConfig {
    pub dt: f32,
    pub n: usize,
    pub theta: f32,
    pub epsilon: f32,
    pub enable_disk_equilibrium_mode: bool,
    pub disk_equilibrium_toomre_q: f32,
    pub disk_equilibrium_scale_height_factor: f32,
    pub disk_equilibrium_asymmetric_drift_strength: f32,
    pub disk_equilibrium_softening_ratio: f32,
    pub disk_equilibrium_theta: f32,
    pub accretion_spawn_rate: f32,
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub outer_ring_spawn_zone_inner_ratio: f32,
    pub outer_ring_spawn_zone_outer_ratio: f32,
    pub central_mass: f32,
    pub particle_mass_range: (f32, f32),
    pub inflow_strength: f32,
    pub restore_strength: f32,
    pub galaxy_separation_factor: f32,
    /// Total number of galaxies to initialize (arranged as pairs; rounded down to an even number).
    pub galaxy_count: usize,
    /// Scales the overall scatter volume in which galaxy pairs are chaotically distributed
    /// (multiplied by pair separation and the cube root of pair count).
    pub galaxy_volume_scatter_factor: f32,
    /// If true, the separation between paired galaxies scales with their estimated masses
    /// (useful for producing wider separations for very massive pairs). The scale uses
    /// galaxy_separation_mass_exponent to determine the power law.
    pub galaxy_mass_weighted_separation: bool,
    /// Exponent applied when mass-weighting separation: separation ∝ (m_total / m_ref)^(exponent)
    pub galaxy_separation_mass_exponent: f32,
    /// Enable coarse cluster grouping of galaxy pairs (sample cluster centers and distribute pairs around them).
    pub galaxy_cluster_enabled: bool,
    /// Number of clusters to create when cluster grouping is enabled. If zero the code will pick a small sensible value.
    pub galaxy_cluster_count: usize,
    /// Radius factor for intra-cluster pair placement relative to scatter volume.
    pub galaxy_cluster_radius_factor: f32,
    /// Strength of the cluster-level bulk motion applied to all pairs inside a cluster.
    pub galaxy_cluster_motion_factor: f32,
    /// Controls how much per-galaxy property variance increases with galaxy_count (higher = more diverse galaxy templates)
    pub galaxy_property_variance_factor: f32,
    /// Fraction of pairs initialized as (near-)circular binaries rather than arbitrary interaction types.
    pub galaxy_binary_circular_fraction: f32,
    /// If true, emit a small debug log of generated cluster/pair centers at initialization.
    pub galaxy_log_centers_on_init: bool,
    pub prob_merge: f32,
    pub prob_repeated: f32,
    pub prob_flyby: f32,
    pub merge_speed_factor: f32,
    pub repeated_speed_factor: f32,
    pub flyby_speed_factor: f32,
    pub merge_angle: f32,
    pub repeated_angle: f32,
    pub flyby_angle: f32,
    pub collision_interval: usize,
    pub attract_interval: usize,
    pub orbital_speed_multiplier: f32,
    pub spawn_angular_speed_base: f32,
    pub spawn_angular_speed_range: f32,
    pub spin_speed_multiplier: f32,
    pub equatorial_growth_factor: f32,
    pub polar_flattening_factor: f32,
    pub radius_scale: f32,
    pub depth_scale_factor: f32,
    pub spacetime_dilation_factor: f32,
    pub adaptive_softening_enabled: bool,
    pub sph_enabled: bool,
    pub softening_scale_factor: f32,
    pub softening_min_factor: f32,
    pub softening_max_factor: f32,
    pub hydro_pressure_strength: f32,
    pub hydro_kernel_factor: f32,
    pub render_update_interval: usize,
    pub performance_mode: bool,
    pub ultra_performance_mode: bool,
    pub enable_galactic_atom_simulation: bool,
    pub enable_elemental_simulation: bool,
    pub enable_elemental_galaxy_pair_mode: bool,
    pub element_atomic_numbers: Vec<u8>,
    pub element_pair_atomic_numbers: Vec<u8>,
    pub element_pair_core_atomic_numbers: Vec<u8>,
    pub enable_chemistry_compass_coloring: bool,
    pub enable_elemental_binding_response: bool,
    pub elemental_binding_response_strength: f32,
    pub element_atomic_count: usize,
    pub element_max_atomic_number: u8,
    pub element_universe_scatter_factor: f32,
    pub element_center_mass_unit: f32,
    pub element_orbital_mass_unit: f32,
    pub element_radius_scale: f32,
    pub element_prime_alpha: f32,
    pub element_prime_beta: f32,
    pub element_light_energy: f32,
    pub enable_elemental_gravitation_tuning: bool,
    pub enable_elemental_mass_dimension_visualization: bool,
    pub element_initial_sorting_mode: usize,
    pub element_initial_grouping_mode: usize,
    pub element_group_spacing: f32,
    pub element_group_internal_velocity_scale: f32,
    pub element_sample_orbit_radius_scale: f32,
    pub element_sample_orbit_speed_factor: f32,
    pub enable_scientific_element_gravity: bool,
    pub element_gravity_mass_weight: f32,
    pub element_gravity_electronegativity_weight: f32,
    pub element_gravity_radius_weight: f32,
    pub element_gravity_reactivity_weight: f32,
    pub enable_molecular_bonding: bool,
    pub molecule_bond_strength_factor: f32,
    pub molecule_break_energy_threshold: f32,
    pub molecule_max_bodies: usize,
    pub molecule_identify_compounds: bool,
    pub gas_enabled: bool,
    pub gas_meta_particle_count: usize,
    pub gas_target_neighbors: usize,
    pub gas_volume_enabled: bool,
    pub gas_volume_count: usize,
    pub gas_volume_radius_factor: f32,
    pub gas_update_interval: usize,
    pub gas_h_min: f32,
    pub gas_h_max: f32,
    pub gas_pressure_coefficient: f32,
    pub gas_condensation_strength: f32,
    pub gas_merge_distance_factor: f32,
    pub gas_merge_velocity_max: f32,
    pub gas_ignition_density: f32,
    pub gas_ignition_temperature: f32,
    pub gas_ignition_mass: f32,
    pub gas_rest_density: f32,
    pub gas_cooling_rate: f32,
    pub gas_relaxation_strength: f32,
    pub gas_render_opacity: f32,
    pub gas_motion_blur_strength: f32,
    pub gas_density_color_scale: f32,
    pub gas_temperature_color_scale: f32,
    pub performance_dynamic_interval_scaling: bool,
    pub performance_body_count_threshold: usize,
    pub performance_max_interval_scale: usize,
    pub performance_render_body_limit: usize,
    pub performance_render_sample_ratio: f32,
    pub total_body_budget: usize,
    pub performance_high_count_mode: bool,
    pub performance_disable_adaptive_updates_in_high_count: bool,
    pub performance_octree_reserve_factor: f32,
    pub enable_adaptive_theta: bool,
    pub enable_adaptive_multipole: bool,
    pub enable_mixed_precision: bool,
    pub enable_adaptive_timestep: bool,
    pub adaptive_theta_quiet: f32,
    pub adaptive_theta_medium: f32,
    pub adaptive_theta_critical: f32,
    pub adaptive_multipole_quiet: usize,
    pub adaptive_multipole_medium: usize,
    pub adaptive_multipole_critical: usize,
    pub adaptive_dynamics_quiet_threshold: f32,
    pub adaptive_dynamics_critical_threshold: f32,
    pub eta_timestep: f32,
    pub dt_min: f32,
    pub dt_max: f32,
    pub max_energy_error: f32,

    // Flow-dynamics feature flag and related defaults (added for GPDM migration)
    pub enable_flow_dynamics_model: bool,
    pub institution_emergence_mode: String,
    pub flow_capacity_mass_per_dt: f32,
    pub flow_capacity_energy_per_dt: f32,
    pub flow_capacity_momentum_per_dt: f32,
    pub flow_capacity_angular_momentum_per_dt: f32,
    pub role_update_interval: usize,
    pub institution_recompute_interval: usize,
    /// Interval (frames) between star evolution updates. Increasing this improves
    /// performance at the cost of star lifecycle fidelity.
    pub star_evolution_interval: usize,
    pub timescale_smbh: usize,
    pub timescale_star_formation: usize,
    pub timescale_galaxy: usize,
    pub timescale_cluster: usize,
}

impl Default for InformationsConfig {
    fn default() -> Self {
        Self {
            dt: 0.02,
            n: 200,
            theta: 1.0,
            epsilon: 1.5,
            enable_disk_equilibrium_mode: false,
            disk_equilibrium_toomre_q: 1.3,
            disk_equilibrium_scale_height_factor: 0.025,
            disk_equilibrium_asymmetric_drift_strength: 0.12,
            disk_equilibrium_softening_ratio: 0.025,
            disk_equilibrium_theta: 0.6,
            accretion_spawn_rate: 0.001,
            inner_radius: 25.0,
            outer_radius: 818.03,
            outer_ring_spawn_zone_inner_ratio: 0.82,
            outer_ring_spawn_zone_outer_ratio: 1.0,
            central_mass: 1e6,
            particle_mass_range: (0.5, 2.0),
            inflow_strength: 0.025,
            restore_strength: 0.08,
            galaxy_separation_factor: 1.0,
            galaxy_count: 2,
            galaxy_volume_scatter_factor: 3.0,
            galaxy_mass_weighted_separation: true,
            galaxy_separation_mass_exponent: 0.333,
            galaxy_cluster_enabled: false,
            galaxy_cluster_count: 4,
            galaxy_cluster_radius_factor: 0.25,
            galaxy_cluster_motion_factor: 0.02,
            galaxy_property_variance_factor: 0.0,
            galaxy_binary_circular_fraction: 0.15,
            galaxy_log_centers_on_init: false,
            prob_merge: 0.35,
            prob_repeated: 0.35,
            prob_flyby: 0.30,
            merge_speed_factor: 0.10,
            repeated_speed_factor: 0.70,
            flyby_speed_factor: 0.90,
            merge_angle: 0.05,
            repeated_angle: 0.25,
            flyby_angle: 0.35,
            collision_interval: 4,
            attract_interval: 2,
            orbital_speed_multiplier: 0.025,
            spawn_angular_speed_base: 0.3,
            spawn_angular_speed_range: 0.5,
            spin_speed_multiplier: 1.0,
            equatorial_growth_factor: 0.18,
            polar_flattening_factor: 0.14,
            radius_scale: 0.25,
            depth_scale_factor: 0.002,
            spacetime_dilation_factor: 2.0,
            adaptive_softening_enabled: true,
            sph_enabled: true,
            softening_scale_factor: 4.0,
            softening_min_factor: 0.5,
            softening_max_factor: 4.0,
            hydro_pressure_strength: 0.02,
            hydro_kernel_factor: 2.0,
            render_update_interval: 2,
            performance_mode: false,
            ultra_performance_mode: false,
            enable_galactic_atom_simulation: false,
            enable_elemental_simulation: false,
            enable_elemental_galaxy_pair_mode: false,
            element_atomic_numbers: Vec::new(),
            element_pair_atomic_numbers: Vec::new(),
            element_pair_core_atomic_numbers: Vec::new(),
            enable_chemistry_compass_coloring: false,
            enable_elemental_binding_response: false,
            elemental_binding_response_strength: 0.0,
            element_atomic_count: 100000,
            element_max_atomic_number: 118,
            element_universe_scatter_factor: 1.8,
            element_center_mass_unit: 1.0,
            element_orbital_mass_unit: 1.0,
            element_radius_scale: 5.0,
            element_prime_alpha: 0.09,
            element_prime_beta: 0.06,
            element_light_energy: 2.0,
            enable_elemental_gravitation_tuning: false,
            enable_elemental_mass_dimension_visualization: false,
            element_initial_sorting_mode: 0,
            element_initial_grouping_mode: 0,
            element_group_spacing: 0.65,
            element_group_internal_velocity_scale: 1.0,
            element_sample_orbit_radius_scale: 1.0,
            element_sample_orbit_speed_factor: 1.0,
            enable_scientific_element_gravity: false,
            element_gravity_mass_weight: 1.0,
            element_gravity_electronegativity_weight: 0.4,
            element_gravity_radius_weight: 0.3,
            element_gravity_reactivity_weight: 0.2,
            enable_molecular_bonding: false,
            molecule_bond_strength_factor: 1.0,
            molecule_break_energy_threshold: 0.8,
            molecule_max_bodies: 12,
            molecule_identify_compounds: false,
            gas_enabled: false,
            gas_meta_particle_count: 100000,
            gas_target_neighbors: 12,
            gas_volume_enabled: false,
            gas_volume_count: 180,
            gas_volume_radius_factor: 0.4,
            gas_update_interval: 2,
            gas_h_min: 0.3,
            gas_h_max: 7.0,
            gas_pressure_coefficient: 0.045,
            gas_condensation_strength: 0.12,
            gas_merge_distance_factor: 1.25,
            gas_merge_velocity_max: 0.14,
            gas_ignition_density: 0.0006,
            gas_ignition_temperature: 0.68,
            gas_ignition_mass: 0.00001,
            gas_rest_density: 0.3,
            gas_cooling_rate: 0.08,
            gas_relaxation_strength: 0.26,
            gas_render_opacity: 0.72,
            gas_motion_blur_strength: 0.38,
            gas_density_color_scale: 1.4,
            gas_temperature_color_scale: 0.9,
            performance_dynamic_interval_scaling: true,
            performance_body_count_threshold: 200000,
            performance_max_interval_scale: 2,
            performance_render_body_limit: 200000,
            performance_render_sample_ratio: 1.0,
            total_body_budget: 200000,
            performance_high_count_mode: false,
            performance_disable_adaptive_updates_in_high_count: false,
            performance_octree_reserve_factor: 4.0,
            enable_adaptive_theta: false,
            enable_adaptive_multipole: false,
            enable_mixed_precision: false,
            enable_adaptive_timestep: false,
            adaptive_theta_quiet: 1.0,
            adaptive_theta_medium: 0.5,
            adaptive_theta_critical: 0.1,
            adaptive_multipole_quiet: 1,
            adaptive_multipole_medium: 2,
            adaptive_multipole_critical: 3,
            adaptive_dynamics_quiet_threshold: 0.001,
            adaptive_dynamics_critical_threshold: 0.35,
            eta_timestep: 1.0,
            dt_min: 0.0,
            dt_max: 0.0,
            max_energy_error: 0.5,

            // GPDM defaults
            enable_flow_dynamics_model: false,
            institution_emergence_mode: "v1".to_string(),
            flow_capacity_mass_per_dt: 1e9,
            flow_capacity_energy_per_dt: 1e9,
            flow_capacity_momentum_per_dt: 1e9,
            flow_capacity_angular_momentum_per_dt: 1e9,
            role_update_interval: 10,
            institution_recompute_interval: 500,
            star_evolution_interval: 10,
            timescale_smbh: 1,
            timescale_star_formation: 100,
            timescale_galaxy: 10000,
            timescale_cluster: 1000000,
        }
    }
}

impl InformationsConfig {
    pub fn load() -> Self {
        // Versuche, den absoluten Pfad zu bestimmen (zuerst im Arbeitsverzeichnis, dann relativ zur Binary)
        // Erstens: relative Pfad-Auflösung (aktuelles Arbeitsverzeichnis)
        let mut possible_paths = vec![PathBuf::from("informations.md")];
        // Allow overriding the informations.md path from the environment for tests and alternate launches.
        if let Ok(override_path) = env::var("INFORMATIONS_MD") {
            possible_paths.insert(0, PathBuf::from(override_path));
        }

        // Wenn die Anwendung als einzelnes Binary ausgeführt wird (z. B. target/release),
        // dann suche nicht nur im Exe-Ordner, sondern steige einige Ebenen nach oben,
        // damit eine informations.md im Projektwurzelverzeichnis gefunden wird.
        if let Ok(exe) = env::current_exe() {
            if let Some(mut dir) = exe.parent() {
                // search up to 5 levels above the exe directory
                let mut levels = 0;
                while levels < 5 {
                    possible_paths.push(dir.join("informations.md"));
                    if let Some(p) = dir.parent() {
                        dir = p;
                    } else {
                        break;
                    }
                    levels += 1;
                }
            }
        }

        let mut file_text = String::new();
        let mut file_found = false;

        for path in &possible_paths {
            if path.exists() {
                match fs::read_to_string(path) {
                    Ok(content) => {
                        file_text = content;
                        file_found = true;
                        if std::env::var("PROFILE_RUNTIME").is_err() {
                        eprintln!("✓ Config geladen: {}", path.display());
                    }
                        break;
                    }
                    Err(e) => {
                        eprintln!("✗ Fehler beim Lesen von {}: {}", path.display(), e);
                    }
                }
            }
        }

        if !file_found {
            if std::env::var("PROFILE_RUNTIME").is_err() {
                eprintln!("⚠ informations.md nicht gefunden. Verwende Standardwerte.");
            }
        }

        let mut config = InformationsConfig::default();
        let mut in_block = false;
        let mut parse_errors = 0;

        let parse_f32 = |value: &str| {
            let cleaned: String = value
                .trim()
                .chars()
                .map(|c| if c == ',' || c == ';' { '.' } else { c })
                .collect();
            cleaned.parse::<f32>().ok()
        };
        let mut primary_atom_mode = None;
        let mut legacy_atom_mode = None;

        // helper: case-insensitive, flexible boolean parser (accepts true/false, yes/no, 1/0, on/off)
        let parse_bool = |s: &str| {
            match s.trim().to_ascii_lowercase().as_str() {
                "true" | "yes" | "on" | "1" => Some(true),
                "false" | "no" | "off" | "0" => Some(false),
                _ => None,
            }
        };

        // helper: flexible usize parsing (accepts integers or floats like "4.0")
        let parse_usize_flexible = |s: &str| {
            let s = s.trim();
            if let Ok(v) = s.parse::<usize>() {
                return Some(v);
            }
            if let Some(f) = parse_f32(s) {
                if f.is_finite() && f >= 0.0 {
                    return Some(f as usize);
                }
            }
            None
        };

        for (line_num, line) in file_text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("```config") {
                in_block = true;
                continue;
            }
            if in_block {
                if trimmed.starts_with("```") {
                    break;
                }
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                let pair = if trimmed.contains('=') {
                    trimmed.split_once('=')
                } else {
                    trimmed.split_once(':')
                };
                if let Some((key, value)) = pair {
                    // allow quoted keys like 'total_body_budget' or "total_body_budget"
                    let key = key.trim_matches('"').trim_matches('\'').trim();
                    let value = value.trim();
                    // remove inline comments ("#" or "//") and surrounding quotes, then trim again
                    let cleaned_value = value
                        .split('#')
                        .next()
                        .unwrap_or(value)
                        .split("//")
                        .next()
                        .unwrap_or(value)
                        .trim();
                    let value = cleaned_value.trim_matches('"').trim_matches('\'').trim();
                    let parse_result = match key {
                        "dt" => parse_f32(value).map(|v| {
                            config.dt = v;
                            true
                        }),
                        "n" => value.parse::<usize>().ok().map(|v| {
                            config.n = v;
                            true
                        }),
                        "theta" => parse_f32(value).map(|v| {
                            config.theta = v;
                            true
                        }),
                        "epsilon" => parse_f32(value).map(|v| {
                            config.epsilon = v;
                            true
                        }),
                        "enable_disk_equilibrium_mode" => parse_bool(value).map(|v| {
                            config.enable_disk_equilibrium_mode = v;
                            true
                        }),
                        "disk_equilibrium_toomre_q" => parse_f32(value).map(|v| {
                            config.disk_equilibrium_toomre_q = v.max(0.0);
                            true
                        }),
                        "disk_equilibrium_scale_height_factor" => parse_f32(value).map(|v| {
                            config.disk_equilibrium_scale_height_factor = v.max(0.0);
                            true
                        }),
                        "disk_equilibrium_asymmetric_drift_strength" => parse_f32(value).map(|v| {
                            config.disk_equilibrium_asymmetric_drift_strength = v.clamp(0.0, 1.0);
                            true
                        }),
                        "disk_equilibrium_softening_ratio" => parse_f32(value).map(|v| {
                            config.disk_equilibrium_softening_ratio = v.max(0.0);
                            true
                        }),
                        "disk_equilibrium_theta" => parse_f32(value).map(|v| {
                            config.disk_equilibrium_theta = v.clamp(0.0, 2.0);
                            true
                        }),
                        "accretion_spawn_rate" => parse_f32(value).map(|v| {
                            config.accretion_spawn_rate = v;
                            true
                        }),
                        "inner_radius" => parse_f32(value).map(|v| {
                            config.inner_radius = v;
                            true
                        }),
                        "outer_radius" => parse_f32(value).map(|v| {
                            config.outer_radius = v;
                            true
                        }),
                        "outer_ring_spawn_zone_inner_ratio" => parse_f32(value).map(|v| {
                            config.outer_ring_spawn_zone_inner_ratio = v;
                            true
                        }),
                        "outer_ring_spawn_zone_outer_ratio" => parse_f32(value).map(|v| {
                            config.outer_ring_spawn_zone_outer_ratio = v;
                            true
                        }),
                        "central_mass" => parse_f32(value).map(|v| {
                            config.central_mass = v;
                            true
                        }),
                        "particle_mass_range" => {
                            let parts: Vec<&str> = value.split(',').map(str::trim).collect();
                            if parts.len() == 2 {
                                if let (Some(min), Some(max)) =
                                    (parse_f32(parts[0]), parse_f32(parts[1]))
                                {
                                    config.particle_mass_range = (min, max);
                                    Some(true)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        "inflow_strength" => parse_f32(value).map(|v| {
                            config.inflow_strength = v;
                            true
                        }),
                        "restore_strength" => parse_f32(value).map(|v| {
                            config.restore_strength = v;
                            true
                        }),
                        "galaxy_separation_factor" => parse_f32(value).map(|v| {
                            config.galaxy_separation_factor = v;
                            true
                        }),
                        "galaxy_count" => value.parse::<usize>().ok().map(|v| {
                            config.galaxy_count = v;
                            true
                        }),
                        "galaxy_volume_scatter_factor" => parse_f32(value).map(|v| {
                            config.galaxy_volume_scatter_factor = v;
                            true
                        }),
                        "galaxy_mass_weighted_separation" => parse_bool(value).map(|v| {
                            config.galaxy_mass_weighted_separation = v;
                            true
                        }),
                        "galaxy_separation_mass_exponent" => parse_f32(value).map(|v| {
                            config.galaxy_separation_mass_exponent = v;
                            true
                        }),
                        "galaxy_cluster_enabled" => parse_bool(value).map(|v| {
                            config.galaxy_cluster_enabled = v;
                            true
                        }),
                        "galaxy_cluster_count" => parse_usize_flexible(value).map(|v| {
                            config.galaxy_cluster_count = v;
                            true
                        }),
                        "galaxy_cluster_radius_factor" => parse_f32(value).map(|v| {
                            config.galaxy_cluster_radius_factor = v;
                            true
                        }),
                        "galaxy_cluster_motion_factor" => parse_f32(value).map(|v| {
                            config.galaxy_cluster_motion_factor = v;
                            true
                        }),
                        "galaxy_property_variance_factor" => parse_f32(value).map(|v| {
                            config.galaxy_property_variance_factor = v;
                            true
                        }),
                        "galaxy_binary_circular_fraction" => parse_f32(value).map(|v| {
                            config.galaxy_binary_circular_fraction = v;
                            true
                        }),
                        "galaxy_log_centers_on_init" => parse_bool(value).map(|v| {
                            config.galaxy_log_centers_on_init = v;
                            true
                        }),
                        "prob_merge" => parse_f32(value).map(|v| {
                            config.prob_merge = v;
                            true
                        }),
                        "prob_repeated" => parse_f32(value).map(|v| {
                            config.prob_repeated = v;
                            true
                        }),
                        "prob_flyby" => parse_f32(value).map(|v| {
                            config.prob_flyby = v;
                            true
                        }),
                        "merge_speed_factor" => parse_f32(value).map(|v| {
                            config.merge_speed_factor = v;
                            true
                        }),
                        "repeated_speed_factor" => parse_f32(value).map(|v| {
                            config.repeated_speed_factor = v;
                            true
                        }),
                        "flyby_speed_factor" => parse_f32(value).map(|v| {
                            config.flyby_speed_factor = v;
                            true
                        }),
                        "merge_angle" => parse_f32(value).map(|v| {
                            config.merge_angle = v;
                            true
                        }),
                        "repeated_angle" => parse_f32(value).map(|v| {
                            config.repeated_angle = v;
                            true
                        }),
                        "flyby_angle" => parse_f32(value).map(|v| {
                            config.flyby_angle = v;
                            true
                        }),
                        "collision_interval" => value.parse::<usize>().ok().map(|v| {
                            config.collision_interval = v;
                            true
                        }),
                        "attract_interval" => value.parse::<usize>().ok().map(|v| {
                            config.attract_interval = v;
                            true
                        }),
                        "orbital_speed_multiplier" => parse_f32(value).map(|v| {
                            config.orbital_speed_multiplier = v;
                            true
                        }),
                        "spawn_angular_speed_base" => parse_f32(value).map(|v| {
                            config.spawn_angular_speed_base = v;
                            true
                        }),
                        "spawn_angular_speed_range" => parse_f32(value).map(|v| {
                            config.spawn_angular_speed_range = v;
                            true
                        }),
                        "spin_speed_multiplier" => parse_f32(value).map(|v| {
                            config.spin_speed_multiplier = v;
                            true
                        }),
                        "equatorial_growth_factor" => parse_f32(value).map(|v| {
                            config.equatorial_growth_factor = v;
                            true
                        }),
                        "polar_flattening_factor" => parse_f32(value).map(|v| {
                            config.polar_flattening_factor = v;
                            true
                        }),
                        "radius_scale" => parse_f32(value).map(|v| {
                            config.radius_scale = v;
                            true
                        }),
                        "depth_scale_factor" => parse_f32(value).map(|v| {
                            config.depth_scale_factor = v;
                            true
                        }),
                        "spacetime_dilation_factor" => parse_f32(value).map(|v| {
                            config.spacetime_dilation_factor = v;
                            true
                        }),
                        "adaptive_softening_enabled" => parse_bool(value).map(|v| {
                            config.adaptive_softening_enabled = v;
                            true
                        }),
                        "sph_enabled" => parse_bool(value).map(|v| {
                            config.sph_enabled = v;
                            true
                        }),
                        "softening_scale_factor" => parse_f32(value).map(|v| {
                            config.softening_scale_factor = v;
                            true
                        }),
                        "softening_min_factor" => parse_f32(value).map(|v| {
                            config.softening_min_factor = v;
                            true
                        }),
                        "softening_max_factor" => parse_f32(value).map(|v| {
                            config.softening_max_factor = v;
                            true
                        }),
                        "hydro_pressure_strength" => parse_f32(value).map(|v| {
                            config.hydro_pressure_strength = v;
                            true
                        }),
                        "hydro_kernel_factor" => parse_f32(value).map(|v| {
                            config.hydro_kernel_factor = v;
                            true
                        }),
                        "render_update_interval" => value.parse::<usize>().ok().map(|v| {
                            config.render_update_interval = v.max(1);
                            true
                        }),
                        "performance_mode" => parse_bool(value).map(|v| {
                            config.performance_mode = v;
                            true
                        }),
                        "ultra_performance_mode" => parse_bool(value).map(|v| {
                            config.ultra_performance_mode = v;
                            true
                        }),
                        "enable_galactic_atom_simulation" => parse_bool(value).map(|v| {
                            primary_atom_mode = Some(v);
                            config.enable_galactic_atom_simulation = v;
                            true
                        }),
                        "enable_elemental_simulation" => parse_bool(value).map(|v| {
                            legacy_atom_mode = Some(v);
                            config.enable_elemental_simulation = v;
                            true
                        }),
                        "enable_elemental_galaxy_pair_mode" => {
                            parse_bool(value).map(|v| {
                                config.enable_elemental_galaxy_pair_mode = v;
                                true
                            })
                        }
                        "element_atomic_numbers" => parse_atomic_numbers(value).map(|numbers| {
                            config.element_atomic_numbers = numbers;
                            true
                        }),
                        "element_pair_atomic_numbers" => {
                            parse_atomic_numbers(value).map(|numbers| {
                                config.element_pair_atomic_numbers = numbers;
                                true
                            })
                        }
                        "element_pair_core_atomic_numbers" => {
                            parse_atomic_numbers(value).map(|numbers| {
                                config.element_pair_core_atomic_numbers = numbers;
                                true
                            })
                        }
                        "enable_chemistry_compass_coloring" => {
                            parse_bool(value).map(|v| {
                                config.enable_chemistry_compass_coloring = v;
                                true
                            })
                        }
                        "enable_elemental_binding_response" => {
                            parse_bool(value).map(|v| {
                                config.enable_elemental_binding_response = v;
                                true
                            })
                        }
                        "elemental_binding_response_strength" => parse_f32(value).map(|v| {
                            config.elemental_binding_response_strength = v.clamp(0.0, 1.0);
                            true
                        }),
                        "element_atomic_count" => value.parse::<usize>().ok().map(|v| {
                            config.element_atomic_count = v.max(1);
                            true
                        }),
                        "element_max_atomic_number" => value.parse::<u8>().ok().map(|v| {
                            config.element_max_atomic_number = v.clamp(1, 118);
                            true
                        }),
                        "element_universe_scatter_factor" => parse_f32(value).map(|v| {
                            config.element_universe_scatter_factor = v.max(0.0);
                            true
                        }),
                        "element_center_mass_unit" => parse_f32(value).map(|v| {
                            config.element_center_mass_unit = v.max(0.0);
                            true
                        }),
                        "element_orbital_mass_unit" => parse_f32(value).map(|v| {
                            config.element_orbital_mass_unit = v.max(0.0);
                            true
                        }),
                        "element_radius_scale" => parse_f32(value).map(|v| {
                            config.element_radius_scale = v.max(0.0);
                            true
                        }),
                        "element_prime_alpha" => parse_f32(value).map(|v| {
                            config.element_prime_alpha = v;
                            true
                        }),
                        "element_prime_beta" => parse_f32(value).map(|v| {
                            config.element_prime_beta = v;
                            true
                        }),
                        "element_light_energy" => parse_f32(value).map(|v| {
                            config.element_light_energy = v.max(0.0);
                            true
                        }),
                        "enable_elemental_gravitation_tuning" => {
                            parse_bool(value).map(|v| {
                                config.enable_elemental_gravitation_tuning = v;
                                true
                            })
                        }
                        "enable_elemental_mass_dimension_visualization" => {
                            parse_bool(value).map(|v| {
                                config.enable_elemental_mass_dimension_visualization = v;
                                true
                            })
                        }
                        "element_initial_sorting_mode" => value.parse::<usize>().ok().map(|v| {
                            config.element_initial_sorting_mode = v.min(2);
                            true
                        }),
                        "element_initial_grouping_mode" => value.parse::<usize>().ok().map(|v| {
                            config.element_initial_grouping_mode = v.min(2);
                            true
                        }),
                        "element_group_spacing" => parse_f32(value).map(|v| {
                            config.element_group_spacing = v.max(0.0);
                            true
                        }),
                        "element_group_internal_velocity_scale" => parse_f32(value).map(|v| {
                            config.element_group_internal_velocity_scale = v.max(0.0);
                            true
                        }),
                        "element_sample_orbit_radius_scale" => parse_f32(value).map(|v| {
                            config.element_sample_orbit_radius_scale = v.max(0.0);
                            true
                        }),
                        "element_sample_orbit_speed_factor" => parse_f32(value).map(|v| {
                            config.element_sample_orbit_speed_factor = v.max(0.0);
                            true
                        }),
                        "enable_scientific_element_gravity" => {
                            value.parse::<bool>().ok().map(|v| {
                                config.enable_scientific_element_gravity = v;
                                true
                            })
                        }
                        "element_gravity_mass_weight" => parse_f32(value).map(|v| {
                            config.element_gravity_mass_weight = v.max(0.0);
                            true
                        }),
                        "element_gravity_electronegativity_weight" => parse_f32(value).map(|v| {
                            config.element_gravity_electronegativity_weight = v.max(0.0);
                            true
                        }),
                        "element_gravity_radius_weight" => parse_f32(value).map(|v| {
                            config.element_gravity_radius_weight = v.max(0.0);
                            true
                        }),
                        "element_gravity_reactivity_weight" => parse_f32(value).map(|v| {
                            config.element_gravity_reactivity_weight = v.max(0.0);
                            true
                        }),
                        "enable_molecular_bonding" => value.parse::<bool>().ok().map(|v| {
                            config.enable_molecular_bonding = v;
                            true
                        }),
                        "molecule_bond_strength_factor" => parse_f32(value).map(|v| {
                            config.molecule_bond_strength_factor = v.clamp(0.0, 5.0);
                            true
                        }),
                        "molecule_break_energy_threshold" => parse_f32(value).map(|v| {
                            config.molecule_break_energy_threshold = v.max(0.0);
                            true
                        }),
                        "molecule_max_bodies" => value.parse::<usize>().ok().map(|v| {
                            config.molecule_max_bodies = v.max(2);
                            true
                        }),
                        "molecule_identify_compounds" => parse_bool(value).map(|v| {
                            config.molecule_identify_compounds = v;
                            true
                        }),
                        "gas_enabled" => parse_bool(value).map(|v| {
                            config.gas_enabled = v;
                            true
                        }),
                        "gas_meta_particle_count" => value.parse::<usize>().ok().map(|v| {
                            config.gas_meta_particle_count = v.max(1);
                            true
                        }),
                        "gas_target_neighbors" => value.parse::<usize>().ok().map(|v| {
                            config.gas_target_neighbors = v.max(1);
                            true
                        }),
                        "gas_volume_enabled" => parse_bool(value).map(|v| {
                            config.gas_volume_enabled = v;
                            true
                        }),
                        "gas_volume_count" => value.parse::<usize>().ok().map(|v| {
                            config.gas_volume_count = v;
                            true
                        }),
                        "gas_volume_radius_factor" => parse_f32(value).map(|v| {
                            config.gas_volume_radius_factor = v.max(0.0);
                            true
                        }),
                        "gas_update_interval" => value.parse::<usize>().ok().map(|v| {
                            config.gas_update_interval = v.max(1);
                            true
                        }),
                        "gas_h_min" => parse_f32(value).map(|v| {
                            config.gas_h_min = v.max(0.0);
                            true
                        }),
                        "gas_h_max" => parse_f32(value).map(|v| {
                            config.gas_h_max = v.max(config.gas_h_min);
                            true
                        }),
                        "gas_pressure_coefficient" => parse_f32(value).map(|v| {
                            config.gas_pressure_coefficient = v.max(0.0);
                            true
                        }),
                        "gas_condensation_strength" => parse_f32(value).map(|v| {
                            config.gas_condensation_strength = v.max(0.0);
                            true
                        }),
                        "gas_merge_distance_factor" => parse_f32(value).map(|v| {
                            config.gas_merge_distance_factor = v.max(0.0);
                            true
                        }),
                        "gas_merge_velocity_max" => parse_f32(value).map(|v| {
                            config.gas_merge_velocity_max = v.max(0.0);
                            true
                        }),
                        "gas_ignition_density" => parse_f32(value).map(|v| {
                            config.gas_ignition_density = v.max(0.0);
                            true
                        }),
                        "gas_ignition_temperature" => parse_f32(value).map(|v| {
                            config.gas_ignition_temperature = v.max(0.0);
                            true
                        }),
                        "gas_ignition_mass" => parse_f32(value).map(|v| {
                            config.gas_ignition_mass = v.max(0.0);
                            true
                        }),
                        "gas_rest_density" => parse_f32(value).map(|v| {
                            config.gas_rest_density = v.max(0.0);
                            true
                        }),
                        "gas_cooling_rate" => parse_f32(value).map(|v| {
                            config.gas_cooling_rate = v.max(0.0);
                            true
                        }),
                        "gas_relaxation_strength" => parse_f32(value).map(|v| {
                            config.gas_relaxation_strength = v.max(0.0);
                            true
                        }),
                        "gas_render_opacity" => parse_f32(value).map(|v| {
                            config.gas_render_opacity = v.clamp(0.0, 1.0);
                            true
                        }),
                        "gas_motion_blur_strength" => parse_f32(value).map(|v| {
                            config.gas_motion_blur_strength = v.clamp(0.0, 1.0);
                            true
                        }),
                        "gas_density_color_scale" => parse_f32(value).map(|v| {
                            config.gas_density_color_scale = v.max(0.0);
                            true
                        }),
                        "gas_temperature_color_scale" => parse_f32(value).map(|v| {
                            config.gas_temperature_color_scale = v.max(0.0);
                            true
                        }),
                        "performance_dynamic_interval_scaling" => {
                            parse_bool(value).map(|v| {
                                config.performance_dynamic_interval_scaling = v;
                                true
                            })
                        }
                        "performance_body_count_threshold" => {
                            value.parse::<usize>().ok().map(|v| {
                                config.performance_body_count_threshold = v.max(1);
                                true
                            })
                        }
                        "performance_max_interval_scale" => value.parse::<usize>().ok().map(|v| {
                            config.performance_max_interval_scale = v.max(1);
                            true
                        }),
                        "performance_render_body_limit" => value.parse::<usize>().ok().map(|v| {
                            config.performance_render_body_limit = v.max(1);
                            true
                        }),
                        "performance_render_sample_ratio" => parse_f32(value).map(|v| {
                            config.performance_render_sample_ratio = v.clamp(0.01, 1.0);
                            true
                        }),
                        "total_body_budget" => parse_usize_flexible(value).map(|v| {
                            config.total_body_budget = v.max(1);
                            true
                        }),
                        "performance_high_count_mode" => parse_bool(value).map(|v| {
                            config.performance_high_count_mode = v;
                            true
                        }),
                        "performance_disable_adaptive_updates_in_high_count" => {
                            parse_bool(value).map(|v| {
                                config.performance_disable_adaptive_updates_in_high_count = v;
                                true
                            })
                        }
                        "performance_octree_reserve_factor" => parse_f32(value).map(|v| {
                            config.performance_octree_reserve_factor = v.max(1.0);
                            true
                        }),
                        "enable_adaptive_theta" => parse_bool(value).map(|v| {
                            config.enable_adaptive_theta = v;
                            true
                        }),
                        "enable_adaptive_multipole" => parse_bool(value).map(|v| {
                            config.enable_adaptive_multipole = v;
                            true
                        }),
                        "enable_mixed_precision" => parse_bool(value).map(|v| {
                            config.enable_mixed_precision = v;
                            true
                        }),
                        "enable_adaptive_timestep" => parse_bool(value).map(|v| {
                            config.enable_adaptive_timestep = v;
                            true
                        }),
                        "adaptive_theta_quiet" => parse_f32(value).map(|v| {
                            config.adaptive_theta_quiet = v.clamp(0.0, 2.0);
                            true
                        }),
                        "adaptive_theta_medium" => parse_f32(value).map(|v| {
                            config.adaptive_theta_medium = v.clamp(0.0, 2.0);
                            true
                        }),
                        "adaptive_theta_critical" => parse_f32(value).map(|v| {
                            config.adaptive_theta_critical = v.clamp(0.0, 2.0);
                            true
                        }),
                        "adaptive_multipole_quiet" => value.parse::<usize>().ok().map(|v| {
                            config.adaptive_multipole_quiet = v.clamp(1, 4);
                            true
                        }),
                        "adaptive_multipole_medium" => value.parse::<usize>().ok().map(|v| {
                            config.adaptive_multipole_medium = v.clamp(1, 4);
                            true
                        }),
                        "adaptive_multipole_critical" => value.parse::<usize>().ok().map(|v| {
                            config.adaptive_multipole_critical = v.clamp(1, 4);
                            true
                        }),
                        "adaptive_dynamics_quiet_threshold" => parse_f32(value).map(|v| {
                            config.adaptive_dynamics_quiet_threshold = v.max(0.0);
                            true
                        }),
                        "adaptive_dynamics_critical_threshold" => parse_f32(value).map(|v| {
                            config.adaptive_dynamics_critical_threshold = v.max(0.0);
                            true
                        }),
                        "eta_timestep" => parse_f32(value).map(|v| {
                            config.eta_timestep = v.max(0.0);
                            true
                        }),
                        "dt_min" => parse_f32(value).map(|v| {
                            config.dt_min = v.max(0.0);
                            true
                        }),
                        "dt_max" => parse_f32(value).map(|v| {
                            config.dt_max = v.max(0.0);
                            true
                        }),
                        "max_energy_error" => parse_f32(value).map(|v| {
                            config.max_energy_error = v.max(0.0);
                            true
                        }),
                        "enable_flow_dynamics_model" => parse_bool(value).map(|v| {
                            config.enable_flow_dynamics_model = v;
                            true
                        }),
                        "institution_emergence_mode" => {
                            // store the raw string (e.g., "v1") for later behavior selection
                            config.institution_emergence_mode = value.to_string();
                            Some(true)
                        }
                        "flow_capacity_mass_per_dt" => parse_f32(value).map(|v| {
                            config.flow_capacity_mass_per_dt = v;
                            true
                        }),
                        "flow_capacity_energy_per_dt" => parse_f32(value).map(|v| {
                            config.flow_capacity_energy_per_dt = v;
                            true
                        }),
                        "flow_capacity_momentum_per_dt" => parse_f32(value).map(|v| {
                            config.flow_capacity_momentum_per_dt = v;
                            true
                        }),
                        "flow_capacity_angular_momentum_per_dt" => parse_f32(value).map(|v| {
                            config.flow_capacity_angular_momentum_per_dt = v;
                            true
                        }),
                        "role_update_interval" => value.parse::<usize>().ok().map(|v| {
                            config.role_update_interval = v;
                            true
                        }),
                        "institution_recompute_interval" => value.parse::<usize>().ok().map(|v| {
                            config.institution_recompute_interval = v;
                            true
                        }),
                        "timescale_smbh" => value.parse::<usize>().ok().map(|v| {
                            config.timescale_smbh = v;
                            true
                        }),
                        "timescale_star_formation" => value.parse::<usize>().ok().map(|v| {
                            config.timescale_star_formation = v;
                            true
                        }),
                        "timescale_galaxy" => value.parse::<usize>().ok().map(|v| {
                            config.timescale_galaxy = v;
                            true
                        }),
                        "timescale_cluster" => value.parse::<usize>().ok().map(|v| {
                            config.timescale_cluster = v;
                            true
                        }),
                        _ => None,
                    };

                    if parse_result.is_none() {
                        eprintln!(
                            "⚠ Config Parse-Fehler Zeile {}: '{}' = '{}' (ungültiger Typ)",
                            line_num + 1,
                            key,
                            value
                        );
                        parse_errors += 1;
                    }
                }
            }
        }

        config.enable_galactic_atom_simulation = resolve_primary_alias(
            primary_atom_mode,
            legacy_atom_mode,
            config.enable_galactic_atom_simulation,
        );
        if config.dt_min == 0.0 {
            config.dt_min = config.dt * 0.1;
        }
        if config.dt_max == 0.0 {
            config.dt_max = config.dt;
        }

        if parse_errors > 0 {
            eprintln!(
                "⚠ {} Parse-Fehler gefunden. Verwende Fallback-Werte für diese.",
                parse_errors
            );
        }

        config
    }
}

fn parse_atomic_numbers(value: &str) -> Option<Vec<u8>> {
    let value = value.trim();
    if value.is_empty() {
        return Some(Vec::new());
    }

    value
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<u8>()
                .ok()
                .filter(|atomic_number| (1..=118).contains(atomic_number))
        })
        .collect()
}

fn resolve_primary_alias(primary: Option<bool>, alias: Option<bool>, default: bool) -> bool {
    primary.or(alias).unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::{parse_atomic_numbers, resolve_primary_alias};

    #[test]
    fn atomic_number_lists_accept_empty_and_valid_lists() {
        assert_eq!(parse_atomic_numbers(""), Some(Vec::new()));
        assert_eq!(parse_atomic_numbers("1, 8,118"), Some(vec![1, 8, 118]));
        assert_eq!(parse_atomic_numbers("0,119"), None);
        assert_eq!(parse_atomic_numbers("1,invalid"), None);
    }

    #[test]
    fn explicit_mode_setting_takes_precedence_over_legacy_alias() {
        assert!(!resolve_primary_alias(Some(false), Some(true), true));
        assert!(resolve_primary_alias(None, Some(true), false));
        assert!(!resolve_primary_alias(None, None, false));
    }

    #[test]
    fn quoted_key_parsing_accepts_quoted_keys() {
        use std::fs::File;
        use std::io::Write;
        // Create a temporary informations.md containing a quoted key/value pair
        let tmp = std::env::temp_dir().join("deadlock_test_informations.md");
        let mut f = File::create(&tmp).expect("create tmp config");
        writeln!(f, "```config").unwrap();
        writeln!(f, "'total_body_budget' = '200000'").unwrap();
        writeln!(f, "```").unwrap();
        // Point loader at our temporary file
        std::env::set_var("INFORMATIONS_MD", &tmp);
        let cfg = super::InformationsConfig::load();
        assert_eq!(cfg.total_body_budget, 200000_usize);
        // Cleanup
        let _ = std::fs::remove_file(&tmp);
    }
}

