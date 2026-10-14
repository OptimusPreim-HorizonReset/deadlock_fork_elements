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
use ultraviolet::Vec3;
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

    // If requested, run a headless render verification pass and exit (useful for CI/debug)
    if std::env::var("VERIFY_RENDER_HEADLESS").is_ok() {
        // Reuse a small sampling of the simulation and run the renderer projection math
        // to check whether any published bodies would be visible on a typical viewport.
        fn verify_render_headless(sim: &mut Simulation, _config: &config::InformationsConfig) {
            use renderer::RenderBody;
            
            use ultraviolet::Vec3;

            // Step the sim a few frames to stabilise
            for _ in 0..3 {
                sim.step();
            }

            // Prepare the published RenderBody vector using same sampling logic as render()
            let limit = sim.performance_render_body_limit();
            let sample_ratio = sim.performance_render_sample_ratio();
            let body_count = sim.bodies.len();
            let mut tmp: Vec<RenderBody> = Vec::new();
            if body_count == 0 {
                eprintln!("VERIFY_RENDER: no bodies in simulation");
                std::process::exit(2);
            }
            if limit >= body_count {
                tmp.reserve(body_count);
                for b in sim.bodies.iter() {
                    tmp.push(RenderBody::from_body(b));
                }
            } else {
                let sample_count = ((limit as f32) * sample_ratio).round() as usize;
                let sample_count = sample_count.max(1).min(limit);
                tmp = Vec::with_capacity(sample_count);
                let stride = (body_count as f32 / sample_count as f32).floor() as usize;
                let stride = stride.max(1);
                let mut idx = 0usize;
                let mut produced = 0usize;
                while produced < sample_count && idx < body_count {
                    tmp.push(RenderBody::from_body(&sim.bodies[idx]));
                    produced += 1;
                    idx += stride;
                }
                let mut j = body_count.saturating_sub(1);
                while produced < sample_count && j > 0 {
                    tmp.push(RenderBody::from_body(&sim.bodies[j]));
                    produced += 1;
                    if j == 0 { break; }
                    j = j.saturating_sub(1);
                }
            }

            // Compute a simple camera autofit like the renderer does
            let sample_count = tmp.len().min(1024);
            let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
            let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
            let mut sum = Vec3::zero();
            for b in tmp.iter().take(sample_count) {
                min.x = min.x.min(b.pos.x);
                min.y = min.y.min(b.pos.y);
                min.z = min.z.min(b.pos.z);
                max.x = max.x.max(b.pos.x);
                max.y = max.y.max(b.pos.y);
                max.z = max.z.max(b.pos.z);
                sum += b.pos;
            }
            let count_f = sample_count as f32;
            let mean = sum / count_f;
            let center = ((min + max) * 0.5 + mean) * 0.5;
            let mut extent = (max - min).mag();
            if !extent.is_finite() || extent <= 0.0 { extent = 80.0; }

            // Compute desired distances (mirror the renderer logic)
            let camera_fov = std::f32::consts::PI / 3.0;
            let tan_half_fov = (camera_fov * 0.5).tan();
            let desired_distance_extent = extent * 1.25;
            let avg_eq_radius = if sample_count > 0 {
                tmp.iter().take(sample_count).map(|b| b.equatorial_radius).sum::<f32>() / sample_count as f32
            } else { 1.0 };
            let depth_scale_est = 0.85_f32;
            let min_visible_radius = 2.0_f32;
            let mut desired_distance_size = if avg_eq_radius > 0.0 {
                (avg_eq_radius * depth_scale_est) / (min_visible_radius * tan_half_fov)
            } else { desired_distance_extent };
            desired_distance_size = desired_distance_size.max(80.0_f32);
            let mut desired_distance = (desired_distance_extent + desired_distance_size * 1.25) * 0.5;
            desired_distance = desired_distance.clamp(80.0, 2_500_000.0);

            // Defensive verify: if none of the sampled bodies would reach a visible pixel radius at the chosen
            // distance, move the camera closer in bounded steps. This mirrors the renderer's autofit adjustments
            // so the headless verifier gives the same practical result as the on-screen renderer.
            let width = 900.0_f32;
            let height = 900.0_f32;
            let view_scale = width.min(height) * 0.5;
            let mut found_visible = false;
            let check_n = sample_count.min(128);
            let camera_yaw = std::f32::consts::PI * 0.25;
            let camera_pitch = -0.25_f32;
            for _attempt in 0..6 {
                let cos_pitch = camera_pitch.cos();
                let cam_pos_test = center + Vec3::new(
                    desired_distance * cos_pitch * camera_yaw.cos(),
                    desired_distance * camera_pitch.sin(),
                    desired_distance * cos_pitch * camera_yaw.sin(),
                );
                let forward_test = (center - cam_pos_test).normalized();
                let _right_test = forward_test.cross(Vec3::new(0.0, 1.0, 0.0)).normalized();

                for b in tmp.iter().take(check_n) {
                    let rel = b.pos - cam_pos_test;
                    let z = rel.dot(forward_test);
                    if z <= 0.01 { continue; }
                    let radius = b.projected_radius() / (z * tan_half_fov).max(0.01);
                    let pixel_radius = radius * view_scale;
                    if pixel_radius >= 1.0 {
                        found_visible = true;
                        break;
                    }
                }

                if found_visible { break; }
                if desired_distance <= 80.0 { break; }
                desired_distance = (desired_distance * 0.5).max(80.0);
            }
            eprintln!("VERIFY_AUTOFIT_ADJUST: final camera_distance={} visible_found={}", desired_distance, found_visible);

            // Build a simple camera from the autofit results and re-implement the projection math
            let camera_fov = std::f32::consts::PI / 3.0;
            let tan_half = (camera_fov * 0.5).tan();
            let camera_yaw = std::f32::consts::PI * 0.25;
            let camera_pitch = -0.25_f32;
            let cos_pitch = camera_pitch.cos();
            let cam_pos = center + Vec3::new(
                desired_distance * cos_pitch * camera_yaw.cos(),
                desired_distance * camera_pitch.sin(),
                desired_distance * cos_pitch * camera_yaw.sin(),
            );
            let forward = (center - cam_pos).normalized();
            let right = forward.cross(Vec3::new(0.0, 1.0, 0.0)).normalized();
            let up = right.cross(forward).normalized();

            // typical viewport for headless check
            let width = 900.0_f32;
            let height = 900.0_f32;
            let aspect = width / height;
            let view_scale = width.min(height) * 0.5;

            // Respect optional env overrides to help debugging remotely
            let force_radius_scale: f32 = std::env::var("RENDER_DEBUG_FORCE_RADIUS_SCALE").ok().and_then(|v| v.parse::<f32>().ok()).unwrap_or(1.0_f32);
            let min_pixel_threshold: f32 = std::env::var("VERIFY_RENDER_MIN_PIXEL").ok().and_then(|v| v.parse::<f32>().ok()).unwrap_or(1.0_f32);

            let mut visible = 0usize;
            for (i, b) in tmp.iter().enumerate() {
                let rel = b.pos - cam_pos;
                let z = rel.dot(forward);
                if z <= 0.1 { continue; }
                let x = rel.dot(right);
                let y = rel.dot(up);
                let proj_x = x / (z * tan_half);
                let proj_y = y / (z * tan_half);
                let ndc_x = proj_x * aspect;
                let ndc_y = proj_y;
                let radius = b.projected_radius() / (z * tan_half).max(0.01);
                let pixel_radius = radius * view_scale * force_radius_scale;
                // If requested, treat the first-N published samples as unculled (matches renderer debug behaviour)
                if let Ok(nstr) = std::env::var("RENDER_DEBUG_DRAW_PUBLISHED_FIRST") {
                    if let Ok(n) = nstr.parse::<usize>() {
                        if i < n {
                            visible += 1;
                            let pixel_center_x = width * 0.5 + ndc_x * view_scale;
                            let pixel_center_y = height * 0.5 + ndc_y * view_scale;
                            eprintln!("VERIFY_RENDER_VISIBLE[{}] (forced) pixel_radius={:.3} pixel=({:.1},{:.1}) world={:?}", i, pixel_radius, pixel_center_x, pixel_center_y, b.pos);
                            continue;
                        }
                    }
                }

                if pixel_radius >= min_pixel_threshold {
                    visible += 1;
                    let pixel_center_x = width * 0.5 + ndc_x * view_scale;
                    let pixel_center_y = height * 0.5 + ndc_y * view_scale;
                    eprintln!("VERIFY_RENDER_VISIBLE[{}] pixel_radius={:.3} pixel=({:.1},{:.1}) world={:?}", i, pixel_radius, pixel_center_x, pixel_center_y, b.pos);
                }
            }
            eprintln!("VERIFY_RENDER: visible_candidates={} / {} sampled", visible, tmp.len());
            if visible == 0 {
                eprintln!("VERIFY_RENDER: no visible published bodies detected (use RENDER_DEBUG_* flags for deeper inspection)");
                std::process::exit(2);
            } else {
                eprintln!("VERIFY_RENDER: success — some bodies project to visible pixel sizes");
                std::process::exit(0);
            }
        }

        verify_render_headless(&mut simulation, &sim_config);
    }

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
            let ts = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                Ok(d) => d.as_secs(),
                Err(err) => {
                    eprintln!("System time before UNIX_EPOCH: {}. Using 0 as timestamp.", err);
                    0u64
                }
            };
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
    hash ^= config.galaxy_count as u64;
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
    if frame.is_multiple_of(interval) {
        {
            // Convert to RenderBody outside of the renderer lock to minimize mutex hold time.
            // To reduce conversion cost at very high body counts, respect the configured
            // performance_render_body_limit and performance_render_sample_ratio by sampling
            // a subset of bodies instead of converting the entire simulation.bodies Vec.
            let body_count = simulation.bodies.len();
            let limit = simulation.performance_render_body_limit().min(body_count);
            // sanitize ratio into [0.0, 1.0]
            let sample_ratio = if simulation.performance_render_sample_ratio() <= 0.0 {
                0.0
            } else if simulation.performance_render_sample_ratio() >= 1.0 {
                1.0
            } else {
                simulation.performance_render_sample_ratio()
            };

            // Optional debug mini-publish mode: publish a small deterministic subset (useful for
            // visual smoke tests when the full scene is too large or camera-autofit misbehaves).
            let debug_mini = std::env::var("RENDER_DEBUG_MINI_PUBLISH").is_ok();

            let mut tmp: Vec<renderer::RenderBody>;
            if debug_mini {
                let sample_count = body_count.min(1024);
                tmp = Vec::with_capacity(sample_count);
                for b in simulation.bodies.iter().take(sample_count) {
                    tmp.push(renderer::RenderBody::from_body(b));
                }
            } else if body_count == 0 || limit == 0 {
                tmp = Vec::new();
            } else if body_count <= limit || sample_ratio >= 1.0 {
                // Convert up to the configured limit (or all if under the limit)
                let cap = body_count.min(limit);
                tmp = Vec::with_capacity(cap);
                for b in simulation.bodies.iter().take(cap) {
                    tmp.push(renderer::RenderBody::from_body(b));
                }
            } else {
                // Sample uniformly across the body array to produce approximately
                // sample_count render entries.
                let sample_count = ((limit as f32) * sample_ratio).round() as usize;
                let sample_count = sample_count.max(1).min(limit);
                tmp = Vec::with_capacity(sample_count);
                let stride = (body_count as f32 / sample_count as f32).floor() as usize;
                let stride = stride.max(1);
                let mut idx = 0usize;
                let mut produced = 0usize;
                while produced < sample_count && idx < body_count {
                    tmp.push(renderer::RenderBody::from_body(&simulation.bodies[idx]));
                    produced += 1;
                    idx += stride;
                }
                // If rounding left us short, fill from the tail until we reach sample_count
                let mut j = body_count.saturating_sub(1);
                while produced < sample_count && j > 0 {
                    tmp.push(renderer::RenderBody::from_body(&simulation.bodies[j]));
                    produced += 1;
                    if j == 0 { break; }
                    j = j.saturating_sub(1);
                }
            }

            let mut lock = renderer::BODIES.lock();
            std::mem::swap(&mut *lock, &mut tmp);
            // tmp (old locked data) is dropped here, releasing the previous buffer without copying.
        }
        {
            let mut lock = renderer::QUADTREE.lock();
            lock.clear();
            lock.extend_from_slice(&simulation.octree.nodes);
        }
        *lock |= true;

        // If requested, compute a forced camera from published bodies and push it to renderer for debugging.
        if std::env::var("RENDER_FORCE_CAMERA").is_ok() {
            let bodies_guard = renderer::BODIES.lock();
            let sample_count = bodies_guard.len().min(128);
            if sample_count > 0 {
                let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
                let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
                let mut sum = Vec3::zero();
                for b in bodies_guard.iter().take(sample_count) {
                    min.x = min.x.min(b.pos.x);
                    min.y = min.y.min(b.pos.y);
                    min.z = min.z.min(b.pos.z);
                    max.x = max.x.max(b.pos.x);
                    max.y = max.y.max(b.pos.y);
                    max.z = max.z.max(b.pos.z);
                    sum += b.pos;
                }
                let count_f = sample_count as f32;
                let mean = sum / count_f;
                let center = ((min + max) * 0.5 + mean) * 0.5;
                let mut extent = (max - min).mag();
                if !extent.is_finite() || extent <= 0.0 { extent = 80.0; }
                let mut desired_distance = (extent * 1.25).max(600.0);
                desired_distance = desired_distance.clamp(80.0, 2_500_000.0);
                *renderer::FORCE_CAMERA_TARGET.lock() = center;
                *renderer::FORCE_CAMERA_DISTANCE.lock() = desired_distance;
                renderer::FORCE_CAMERA.store(true, Ordering::Relaxed);
            }
        }

        // Diagnostic log to confirm publishing into shared renderer buffers
        eprintln!(
            "PUBLISH: frame={} bodies={} quadtree_nodes={}",
            frame,
            renderer::BODIES.lock().len(),
            renderer::QUADTREE.lock().len()
        );

        // Optional debug dump of first few published positions to help trace visibility issues
        if std::env::var("RENDER_DEBUG_LOG_POSITIONS").is_ok() {
            let bodies_guard = renderer::BODIES.lock();
            for (i, b) in bodies_guard.iter().take(8).enumerate() {
                eprintln!("PUBLISH_POS[{}]: {:?}", i, b.pos);
            }
        }
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
