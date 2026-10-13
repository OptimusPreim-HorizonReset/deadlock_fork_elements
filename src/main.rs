use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

mod accounts;
mod body;
mod config;
mod contact_parliament;
mod elements;
mod emergence;
mod galaxy;
mod galaxy_templates;
mod galaxy_tree;
mod gas;
mod history;
mod id_alloc;
mod molecules;
mod quadtree;
mod render_mode;
mod renderer;
mod astro;
mod roles;
mod simulation;
mod tables;
mod timescales;
mod utils;

#[cfg(test)]
mod scientific_validation;

use renderer::Renderer;
use simulation::Simulation;

fn main() {
    let app_config = quarkstrom::Config {
        window_mode: quarkstrom::WindowMode::Windowed(900, 900),
    };

    let sim_config = config::InformationsConfig::load();
    renderer::CHEMISTRY_COMPASS_COLORING.store(
        sim_config.enable_chemistry_compass_coloring,
        Ordering::Relaxed,
    );

    // Optional: respect profiling max-frames via env var `PROFILE_MAX_FRAMES`
    // If set to a valid usize, the spawned simulation thread will exit the process
    // after the frame count reaches this value. Useful for automated profiling runs.
    let profile_max_frames: Option<usize> = std::env::var("PROFILE_MAX_FRAMES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok());
    let profile_runtime: bool = std::env::var("PROFILE_RUNTIME")
        .ok()
        .map(|v| v != "0")
        .unwrap_or(false);
    let profile_frames: usize = std::env::var("PROFILE_FRAMES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(50);
    let profile_step_components: bool = std::env::var("PROFILE_STEP_COMPONENTS")
        .ok()
        .map(|v| v != "0")
        .unwrap_or(false);

    if std::env::args().any(|arg| arg == "--headless-test") || std::env::var("HEADLESS_SIM").is_ok()
    {
        run_headless_test(&sim_config);
        return;
    }

    let mut simulation = Simulation::new(&sim_config);
    // One-shot recompute on initial startup if flow-dynamics feature is enabled
    if sim_config.enable_flow_dynamics_model {
        simulation.cache_galaxies();
        simulation.recompute_institutions();
        eprintln!(
            "🟦 One-shot emergence recompute executed at startup ({} tables)",
            crate::tables::TABLES.lock().len()
        );
        // Request renderer to re-run camera autofit so new scene scale is visible
        renderer::CAMERA_AUTOFIT_REQUEST.store(true, Ordering::Relaxed);
    }
    let last_config_load = Instant::now();
    let config_reload_interval = Duration::from_secs(1); // Alle 1 Sekunde versuchen neu zu laden

    std::thread::spawn(move || {
        let mut simulation = simulation;
        let mut last_config_load = last_config_load;
        let mut last_config_hash = config_hash(&sim_config);
        // Profiling accumulators
        let mut step_time_acc: u128 = 0;
        let mut render_time_acc: u128 = 0;
        let mut frames_in_cycle: usize = 0;
        let mut component_acc: HashMap<String, u128> = HashMap::new();
        // Optional CSV logger for periodic profiler dumps
        let mut csv_file: Option<File> = None;
        if profile_runtime {
            // ensure logs folder exists
            let _ = std::fs::create_dir_all("logs");
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let filename = format!("logs/profiler-{}.csv", ts);
            match File::create(&filename) {
                Ok(mut f) => {
                    let header = "frame_start,frame_end,avg_step_ms,avg_render_ms,component,component_avg_ms\n";
                    let _ = f.write_all(header.as_bytes());
                    csv_file = Some(f);
                    eprintln!("🗃️ Profiling CSV: {}", filename);
                }
                Err(err) => {
                    eprintln!("Failed to create profiler CSV file {}: {}", filename, err);
                }
            }
        }

        loop {
            if renderer::PAUSED.load(Ordering::Relaxed) {
                std::thread::yield_now();
            } else {
                // Kontinuierliches Reload: Versuche Config zu laden wenn genug Zeit vergangen ist
                if last_config_load.elapsed() > config_reload_interval {
                    let new_config = config::InformationsConfig::load();
                    let new_hash = config_hash(&new_config);
                    if new_hash != last_config_hash {
                        eprintln!("🔄 Config-Änderung erkannt! Starte Simulation neu...");
                        renderer::CHEMISTRY_COMPASS_COLORING.store(
                            new_config.enable_chemistry_compass_coloring,
                            Ordering::Relaxed,
                        );
                        simulation = Simulation::new(&new_config);
                        // Clear any debug-only registries so visualizations do not accumulate stale entries
                        crate::tables::TABLES.lock().clear();
                        // One-shot recompute of emergent institutions for immediate visualization if enabled
                        if new_config.enable_flow_dynamics_model {
                            // refresh galaxy snapshots and compute emergent tables
                            simulation.cache_galaxies();
                            simulation.recompute_institutions();
                            eprintln!(
                                "🟦 One-shot emergence recompute executed after config reload ({} tables)",
                                crate::tables::TABLES.lock().len()
                            );
                            // Request renderer to re-run camera autofit due to possible scale/config change
                            renderer::CAMERA_AUTOFIT_REQUEST.store(true, Ordering::Relaxed);
                        }
                        last_config_hash = new_hash;
                    }
                    last_config_load = Instant::now();
                }

                // Alt: RESET_REQUESTED Flag (für Benutzer-Aktion)
                if renderer::RESET_REQUESTED.swap(false, Ordering::Relaxed) {
                    let sim_config = config::InformationsConfig::load();
                    simulation = Simulation::new(&sim_config);
                    // Clear debug registries on explicit reset as well
                    crate::tables::TABLES.lock().clear();
                    // One-shot recompute so Organigram shows emergent tables immediately
                    if sim_config.enable_flow_dynamics_model {
                        simulation.cache_galaxies();
                        simulation.recompute_institutions();
                        eprintln!(
                            "🟦 One-shot emergence recompute executed after UI Reset ({} tables)",
                            crate::tables::TABLES.lock().len()
                        );
                        // Request renderer to re-run camera autofit after explicit reset
                        renderer::CAMERA_AUTOFIT_REQUEST.store(true, Ordering::Relaxed);
                    }
                    eprintln!("🔄 Reset via UI angefordert!");
                }

                if profile_runtime {
                    // Profile step
                    let step_start = Instant::now();
                    if profile_step_components {
                        // Collect component timings from simulation
                        let comps = simulation.step_profiled();
                        let step_micros: u128 = comps.iter().map(|(_, v)| *v).sum();
                        for (name, dur) in comps {
                            *component_acc.entry(name).or_insert(0) += dur;
                        }
                        step_time_acc += step_micros;
                    } else {
                        simulation.step();
                        step_time_acc += step_start.elapsed().as_micros();
                    }

                    let frame = simulation.frame;

                    // Profile render
                    let render_start = Instant::now();
                    render(&mut simulation, frame);
                    render_time_acc += render_start.elapsed().as_micros();

                    frames_in_cycle += 1;
                    if frames_in_cycle >= profile_frames {
                        let avg_step_ms = (step_time_acc as f64) / (frames_in_cycle as f64) / 1000.0;
                        let avg_render_ms = (render_time_acc as f64) / (frames_in_cycle as f64) / 1000.0;
                        eprintln!(
                            "📊 Profiling (last {} frames): avg step = {:#.3} ms, avg render = {:#.3} ms",
                            frames_in_cycle, avg_step_ms, avg_render_ms
                        );
                        if profile_step_components {
                            eprintln!("   Component averages (ms):");
                            for (name, tot) in component_acc.iter() {
                                let avg = (*tot as f64) / (frames_in_cycle as f64) / 1000.0;
                                eprintln!("    - {}: {:#.3}", name, avg);
                            }
                        }
                        // Persist profiler summary and per-component averages to CSV if enabled
                        if let Some(f) = csv_file.as_mut() {
                            let frame_end = simulation.frame;
                            let frame_start = frame_end.saturating_sub(frames_in_cycle.saturating_sub(1));
                            let _ = writeln!(f, "{},{},{:.6},{:.6},,", frame_start, frame_end, avg_step_ms, avg_render_ms);
                            if profile_step_components {
                                for (name, tot) in component_acc.iter() {
                                    let comp_avg = (*tot as f64) / (frames_in_cycle as f64) / 1000.0;
                                    let _ = writeln!(f, "{},{},{:.6},{:.6},{},{:.6}", frame_start, frame_end, avg_step_ms, avg_render_ms, name, comp_avg);
                                }
                            }
                            let _ = f.flush();
                        }
                        // reset
                        step_time_acc = 0;
                        render_time_acc = 0;
                        frames_in_cycle = 0;
                        component_acc.clear();
                    }

                    // Wenn PROFILE_MAX_FRAMES gesetzt ist, beenden wir den Prozess nachdem
                    // diese Frame-Anzahl erreicht wurde (nützlich für automatisches Profiling).
                    if let Some(max) = profile_max_frames {
                        if frame >= max {
                            eprintln!("🧪 PROFILE_MAX_FRAMES reached: {} — exiting.", max);
                            std::process::exit(0);
                        }
                    }
                } else {
                    // Normaler Pfad ohne Laufzeitprofiling
                    simulation.step();
                    let frame = simulation.frame;
                    render(&mut simulation, frame);
                    if let Some(max) = profile_max_frames {
                        if frame >= max {
                            eprintln!("🧪 PROFILE_MAX_FRAMES reached: {} — exiting.", max);
                            std::process::exit(0);
                        }
                    }
                }
            }
        }
    });

    quarkstrom::run::<Renderer>(app_config);
}

/// Berechnet einen einfachen Hash der Config für Änderungserkennung
fn config_hash(config: &config::InformationsConfig) -> u64 {
    let mut hash: u64 = 0;
    hash ^= config.dt.to_bits() as u64;
    hash ^= config.n as u64;
    hash ^= config.theta.to_bits() as u64;
    hash ^= config.epsilon.to_bits() as u64;
    hash ^= config.enable_disk_equilibrium_mode as u64;
    hash ^= config.disk_equilibrium_toomre_q.to_bits() as u64;
    hash ^= config.disk_equilibrium_scale_height_factor.to_bits() as u64;
    hash ^= config.disk_equilibrium_asymmetric_drift_strength.to_bits() as u64;
    hash ^= config.disk_equilibrium_softening_ratio.to_bits() as u64;
    hash ^= config.disk_equilibrium_theta.to_bits() as u64;
    hash ^= config.outer_radius.to_bits() as u64;
    hash ^= config.inner_radius.to_bits() as u64;
    hash ^= config.galaxy_separation_factor.to_bits() as u64;
    hash ^= config.accretion_spawn_rate.to_bits() as u64;
    hash ^= config.central_mass.to_bits() as u64;
    hash ^= config.collision_interval as u64;
    hash ^= config.attract_interval as u64;
    hash ^= config.render_update_interval as u64;
    hash ^= config.spacetime_dilation_factor.to_bits() as u64;
    hash ^= config.performance_mode as u64;
    hash ^= config.ultra_performance_mode as u64;
    hash ^= config.adaptive_softening_enabled as u64;
    hash ^= config.sph_enabled as u64;
    hash ^= config.softening_scale_factor.to_bits() as u64;
    hash ^= config.softening_min_factor.to_bits() as u64;
    hash ^= config.softening_max_factor.to_bits() as u64;
    hash ^= config.hydro_pressure_strength.to_bits() as u64;
    hash ^= config.hydro_kernel_factor.to_bits() as u64;
    hash ^= config.enable_galactic_atom_simulation as u64;
    hash ^= config.enable_elemental_simulation as u64;
    hash ^= config.enable_elemental_galaxy_pair_mode as u64;
    hash ^= config.enable_chemistry_compass_coloring as u64;
    hash ^= config.enable_elemental_binding_response as u64;
    hash ^= config.elemental_binding_response_strength.to_bits() as u64;
    hash ^= config.element_atomic_count as u64;
    hash ^= config.element_max_atomic_number as u64;
    hash ^= config.element_universe_scatter_factor.to_bits() as u64;
    hash ^= config.element_center_mass_unit.to_bits() as u64;
    hash ^= config.element_orbital_mass_unit.to_bits() as u64;
    hash ^= config.element_radius_scale.to_bits() as u64;
    hash ^= config.element_prime_alpha.to_bits() as u64;
    hash ^= config.element_prime_beta.to_bits() as u64;
    hash ^= config.element_light_energy.to_bits() as u64;
    hash ^= config.enable_elemental_gravitation_tuning as u64;
    hash ^= config.enable_elemental_mass_dimension_visualization as u64;
    hash ^= config.element_initial_sorting_mode as u64;
    hash ^= config.element_initial_grouping_mode as u64;
    hash ^= config.element_group_spacing.to_bits() as u64;
    hash ^= config.element_group_internal_velocity_scale.to_bits() as u64;
    hash ^= config.element_sample_orbit_radius_scale.to_bits() as u64;
    hash ^= config.element_sample_orbit_speed_factor.to_bits() as u64;
    hash ^= config.enable_scientific_element_gravity as u64;
    hash ^= config.element_gravity_mass_weight.to_bits() as u64;
    hash ^= config.element_gravity_electronegativity_weight.to_bits() as u64;
    hash ^= config.element_gravity_radius_weight.to_bits() as u64;
    hash ^= config.element_gravity_reactivity_weight.to_bits() as u64;
    hash ^= config.enable_molecular_bonding as u64;
    hash ^= config.molecule_bond_strength_factor.to_bits() as u64;
    hash ^= config.molecule_break_energy_threshold.to_bits() as u64;
    hash ^= config.molecule_max_bodies as u64;
    hash ^= config.molecule_identify_compounds as u64;
    hash ^= config.gas_enabled as u64;
    hash ^= config.gas_meta_particle_count as u64;
    hash ^= config.gas_target_neighbors as u64;
    hash ^= config.gas_volume_enabled as u64;
    hash ^= config.gas_volume_count as u64;
    hash ^= config.gas_volume_radius_factor.to_bits() as u64;
    hash ^= config.gas_update_interval as u64;
    hash ^= config.gas_h_min.to_bits() as u64;
    hash ^= config.gas_h_max.to_bits() as u64;
    hash ^= config.gas_pressure_coefficient.to_bits() as u64;
    hash ^= config.gas_condensation_strength.to_bits() as u64;
    hash ^= config.gas_merge_distance_factor.to_bits() as u64;
    hash ^= config.gas_merge_velocity_max.to_bits() as u64;
    hash ^= config.gas_ignition_density.to_bits() as u64;
    hash ^= config.gas_ignition_temperature.to_bits() as u64;
    hash ^= config.gas_ignition_mass.to_bits() as u64;
    hash ^= config.gas_rest_density.to_bits() as u64;
    hash ^= config.gas_cooling_rate.to_bits() as u64;
    hash ^= config.gas_relaxation_strength.to_bits() as u64;
    hash ^= config.gas_render_opacity.to_bits() as u64;
    hash ^= config.gas_motion_blur_strength.to_bits() as u64;
    hash ^= config.gas_density_color_scale.to_bits() as u64;
    hash ^= config.gas_temperature_color_scale.to_bits() as u64;
    hash ^= config.performance_dynamic_interval_scaling as u64;
    hash ^= config.performance_body_count_threshold as u64;
    hash ^= config.performance_max_interval_scale as u64;
    hash ^= config.performance_render_body_limit as u64;
    hash ^= config.performance_render_sample_ratio.to_bits() as u64;
    hash ^= config.performance_high_count_mode as u64;
    hash ^= config.performance_disable_adaptive_updates_in_high_count as u64;
    hash ^= config.performance_octree_reserve_factor.to_bits() as u64;
    hash ^= config.enable_adaptive_theta as u64;
    hash ^= config.enable_adaptive_multipole as u64;
    hash ^= config.enable_mixed_precision as u64;
    hash ^= config.enable_adaptive_timestep as u64;
    hash ^= config.adaptive_theta_quiet.to_bits() as u64;
    hash ^= config.adaptive_theta_medium.to_bits() as u64;
    hash ^= config.adaptive_theta_critical.to_bits() as u64;
    hash ^= config.adaptive_multipole_quiet as u64;
    hash ^= config.adaptive_multipole_medium as u64;
    hash ^= config.adaptive_multipole_critical as u64;
    hash ^= config.adaptive_dynamics_quiet_threshold.to_bits() as u64;
    hash ^= config.adaptive_dynamics_critical_threshold.to_bits() as u64;
    hash ^= config.eta_timestep.to_bits() as u64;
    hash ^= config.dt_min.to_bits() as u64;
    hash ^= config.dt_max.to_bits() as u64;
    hash ^= config.max_energy_error.to_bits() as u64;
    for atomic_number in &config.element_atomic_numbers {
        hash = hash.rotate_left(5) ^ *atomic_number as u64;
    }
    for atomic_number in &config.element_pair_atomic_numbers {
        hash = hash.rotate_left(5) ^ *atomic_number as u64;
    }
    for atomic_number in &config.element_pair_core_atomic_numbers {
        hash = hash.rotate_left(5) ^ *atomic_number as u64;
    }
    hash
}

fn render(simulation: &mut Simulation, frame: usize) {
    let interval = simulation.render_update_interval();
    let mut lock = renderer::UPDATE_LOCK.lock();
    if frame % interval == 0 {
        {
            let mut lock = renderer::BODIES.lock();
            lock.clear();
            lock.extend_from_slice(&simulation.bodies);
        }
        {
            let mut lock = renderer::QUADTREE.lock();
            lock.clear();
            lock.extend_from_slice(&simulation.octree.nodes);
        }
        *lock |= true;
        // Diagnostic log to confirm publishing into shared renderer buffers
        eprintln!(
            "PUBLISH: frame={} bodies={} quadtree_nodes={}",
            frame,
            renderer::BODIES.lock().len(),
            renderer::QUADTREE.lock().len()
        );
    }
}

fn run_headless_test(config: &config::InformationsConfig) {
    let mut test_config = config.clone();
    test_config.n = 20;
    test_config.galaxy_count = 2;
    let mut simulation = Simulation::new(&test_config);
    let body_count = simulation.bodies.len();
    println!(
        "Headless particle movement test ({} galaxies, {} particles each): {} bodies",
        test_config.galaxy_count, test_config.n, body_count
    );

    let before_positions: Vec<_> = simulation.bodies.iter().map(|body| body.pos).collect();
    let steps = 20;
    for step in 0..steps {
        simulation.step();
        if step == 0 || step + 1 == steps {
            let displacement = simulation
                .bodies
                .iter()
                .zip(before_positions.iter())
                .map(|(body, before_pos)| (body.pos - *before_pos).mag())
                .fold(0.0_f32, |sum, d| sum + d);
            println!(
                " step {:>2}: total displacement = {:.6}",
                step + 1,
                displacement
            );
        }
    }

    let moved_count = simulation
        .bodies
        .iter()
        .zip(before_positions.iter())
        .filter(|(body, before_pos)| body.pos != **before_pos)
        .count();

    println!(
        "Particle movement verification: {} / {} moved",
        moved_count, body_count
    );
    if moved_count == 0 {
        eprintln!("ERROR: no particles moved during simulation steps.");
        std::process::exit(1);
    }
}
