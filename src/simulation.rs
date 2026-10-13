#![allow(dead_code, clippy::manual_is_multiple_of, clippy::collapsible_if, clippy::field_reassign_with_default)]

use crate::{
    body::{Body, BodySegment},
    config::InformationsConfig,
    elements::{all_elements, element_by_atomic_number},
    elements::{binding_tendency, BindingTendency},
    galaxy_templates::{plummer_circular_speed, GalaxyTemplate},
    molecules::MolecularBond,
    quadtree::{Oct, Octree},
    renderer,
};

use crate::galaxy::GalaxyState;
use crate::timescales::TimescaleScheduler;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use ultraviolet::{Vec2, Vec3};

pub struct Simulation {
    pub dt: f32,
    pub frame: usize,
    pub bodies: Vec<Body>,
    pub octree: Octree,
    /// Cached per-galaxy snapshots (GalaxyState) for emergence and institution workflows
    pub galaxies: Vec<GalaxyState>,
    /// Per-galaxy start indices inside the bodies Vec for slicing into GalaxyState snapshots.
    galaxy_start_indices: Vec<usize>,
    /// Template used to derive accretion spawn parameters.
    accretion_template: GalaxyTemplate,
    config: InformationsConfig,
    /// Index of first galaxy center
    center1_idx: usize,
    /// Index of second galaxy center
    center2_idx: usize,
    /// Resulting equatorial plane normal for hydrodynamic inflow alignment.
    equatorial_plane_normal: Vec3,
    /// Position through which the equatorial plane passes.
    equatorial_plane_center: Vec3,
    // Cluster motion persisted state (per-cluster components and current velocities)
    cluster_base_translations: Vec<Vec3>,
    cluster_orbit_bases: Vec<Vec3>,
    cluster_phases: Vec<f32>,
    cluster_freqs: Vec<f32>,
    /// Current per-cluster velocity applied each frame (updated by apply_cluster_motion)
    cluster_velocities: Vec<Vec3>,
    /// Mapping from body index -> cluster id assigned at spawn time
    body_cluster_ids: Vec<usize>,
    next_body_id: u64,
    next_molecule_id: u64,
    molecular_bonds: Vec<MolecularBond>,
    /// Timescale scheduler for low-frequency updates (roles, institutions, etc.)
    timescale_scheduler: TimescaleScheduler,
}

impl Simulation {
    pub fn new(config: &InformationsConfig) -> Self {
        let dt = config.dt;
        let theta = config.theta;
        let epsilon = config.epsilon;

        let accretion_template = GalaxyTemplate::from_config(config);
        if config.enable_galactic_atom_simulation {
            return Self::new_galactic_atom_simulation(
                config,
                accretion_template,
                dt,
                theta,
                epsilon,
            );
        }
        let config = config.clone();

        // Determine how many galaxy pairs to create. The config.galaxy_count is interpreted
        // as total galaxies arranged in pairs; round down to an even number of galaxies.
        let pair_count = (config.galaxy_count / 2).max(1);
        let total_galaxies = pair_count * 2;

        // Scatter scale: scale with outer_radius and configured factor and the cube-root of pair count
        let scatter_scale_max = accretion_template.outer_radius
            * config.galaxy_volume_scatter_factor
            * (pair_count as f32).powf(1.0 / 3.0);

        let _separation = accretion_template.outer_radius * config.galaxy_separation_factor;

        // Pre-compute elemental pairing lists if the mode is enabled (reuse across pairs)
        let (atomic_numbers, core_candidates) = if config.enable_elemental_galaxy_pair_mode && config.n > 200 {
            let nums = if config.element_pair_atomic_numbers.is_empty() {
                all_elements().map(|element| element.atomic_number).collect::<Vec<_>>()
            } else {
                config.element_pair_atomic_numbers.clone()
            };
            let cores = if config.element_pair_core_atomic_numbers.is_empty() {
                nums.clone()
            } else {
                config.element_pair_core_atomic_numbers.clone()
            };
            (nums, cores)
        } else {
            (Vec::new(), Vec::new())
        };

        let mut bodies: Vec<Body> = Vec::new();
        let mut galaxy_start_indices: Vec<usize> = Vec::new();
        let mut body_cluster_ids: Vec<usize> = Vec::new();

        // Property variance increases with galaxy_count to produce more diverse morphologies
        let property_variance_scale = 1.0 + config.galaxy_property_variance_factor * (config.galaxy_count as f32 / 50.0);

        // Auto-enable clustering for larger galaxy counts, or obey explicit flag.
        let cluster_enabled = config.galaxy_cluster_enabled || config.galaxy_count >= 50;
        let cluster_count = if cluster_enabled {
            config.galaxy_cluster_count.max(1).min(pair_count)
        } else {
            1usize
        };

        // Sample cluster centers (or a single global center) inside the scatter sphere.
        let mut cluster_centers: Vec<Vec3> = Vec::with_capacity(cluster_count);
        for _ in 0..cluster_count {
            let u = fastrand::f32();
            let r = scatter_scale_max * u.powf(1.0 / 3.0) * 0.6;
            let cos_theta = 2.0 * fastrand::f32() - 1.0;
            let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
            let phi = fastrand::f32() * std::f32::consts::TAU;
            let cx = r * sin_theta * phi.cos();
            let cy = r * sin_theta * phi.sin();
            let cz = r * cos_theta;
            cluster_centers.push(Vec3::new(cx, cy, cz));
        }

        // Distribute pairs among clusters as evenly as possible.
        let mut pairs_per_cluster = vec![pair_count / cluster_count; cluster_count];
        for i in 0..(pair_count % cluster_count) {
            pairs_per_cluster[i] += 1;
        }
        let mut assigned_in_cluster = vec![0usize; cluster_count];

        // Precompute a reference mass for separation scaling heuristics.
        let avg_template_particle_mass = (accretion_template.particle_mass_range.0 + accretion_template.particle_mass_range.1) * 0.5;
        let mass_ref = accretion_template.central_mass + accretion_template.n as f32 * avg_template_particle_mass;

        // Global total body budget handling.
        // Reserve one slot per galaxy for the central mass; the remaining budget is available for disk particles.
        // If total_body_budget == 0 then no budget constraint is applied.
        let (mut remaining_disk_budget, mut remaining_galaxy_slots): (isize, usize) = if config.total_body_budget > 0 {
            let reserved_centers = total_galaxies as usize;
            let available = config.total_body_budget.saturating_sub(reserved_centers);
            (available as isize, total_galaxies)
        } else {
            (-1isize, total_galaxies)
        };

        // Precompute coherent cluster motion components.
        // We store a small base translation and an orbital base vector for each cluster,
        // plus randomized phase/frequency so the cluster velocities evolve over time.
        let mut cluster_base_translations: Vec<Vec3> = Vec::with_capacity(cluster_count);
        let mut cluster_orbit_bases: Vec<Vec3> = Vec::with_capacity(cluster_count);
        let mut cluster_phases: Vec<f32> = Vec::with_capacity(cluster_count);
        let mut cluster_freqs: Vec<f32> = Vec::with_capacity(cluster_count);
        let mut cluster_init_vels: Vec<Vec3> = Vec::with_capacity(cluster_count);
        for c in 0..cluster_count {
            let cc = cluster_centers[c];
            let r = cc.mag().max(1e-4);
            // estimate cluster mass (approx. per-galaxy mass * number of galaxies in cluster)
            let galaxies_in_cluster = (pairs_per_cluster[c] * 2) as f32;
            let estimated_cluster_mass = mass_ref * galaxies_in_cluster;
            // tangential direction around world origin (coherent swirling)
            let reference = if cc.abs().x < 0.9 { Vec3::new(1.0, 0.0, 0.0) } else { Vec3::new(0.0, 1.0, 0.0) };
            let mut tangent = cc.cross(reference);
            if tangent.mag_sq() <= 1e-8 {
                tangent = Vec3::new(-cc.y, cc.x, 0.0);
            }
            tangent = tangent.normalized();
            // orbital base vector (unscaled)
            let v_orbit_base = (estimated_cluster_mass / r).sqrt() * tangent * 0.5;
            // small coherent translation for natural drift
            let base_translation = Vec3::new(
                (fastrand::f32() - 0.5) * 0.2,
                (fastrand::f32() - 0.5) * 0.2,
                (fastrand::f32() - 0.5) * 0.1,
            ) * (scatter_scale_max * 0.005);
            // randomize phase and slow frequency so evolution is visible but gradual
            let phase = fastrand::f32() * std::f32::consts::TAU;
            let freq = 0.0005 + fastrand::f32() * 0.002;
            // initial velocity sample (used for initial assignment)
            let init_v = (base_translation + v_orbit_base) * config.galaxy_cluster_motion_factor * (0.4 + fastrand::f32() * 0.6);

            cluster_base_translations.push(base_translation);
            cluster_orbit_bases.push(v_orbit_base);
            cluster_phases.push(phase);
            cluster_freqs.push(freq);
            cluster_init_vels.push(init_v);
        }

        let mut pair_centers_preview: Vec<Vec3> = Vec::new();
        for _pair_index in 0..pair_count {
            // choose which cluster this pair belongs to (first-fit)
            let mut cid = 0usize;
            for c in 0..cluster_count {
                if assigned_in_cluster[c] < pairs_per_cluster[c] {
                    cid = c;
                    break;
                }
            }
            assigned_in_cluster[cid] += 1;

            // sample a local offset around the cluster center to get the pair center
            let cluster_center = cluster_centers[cid];
            let u = fastrand::f32();
            let local_r = scatter_scale_max * config.galaxy_cluster_radius_factor * (pairs_per_cluster[cid] as f32).powf(1.0 / 3.0) * u.powf(1.0 / 3.0);
            let cos_theta = 2.0 * fastrand::f32() - 1.0;
            let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
            let phi = fastrand::f32() * std::f32::consts::TAU;
            let ox = local_r * sin_theta * phi.cos();
            let oy = local_r * sin_theta * phi.sin();
            let oz = local_r * cos_theta;
            let pair_center = cluster_center + Vec3::new(ox, oy, oz);
            if pair_centers_preview.len() < 8 {
                pair_centers_preview.push(pair_center);
            }

            // choose a (numerically robust) random separation direction
            let mut dir = Vec3::new(
                fastrand::f32() - 0.5,
                fastrand::f32() - 0.5,
                fastrand::f32() - 0.5,
            );
            if dir.mag_sq() <= 1e-8 {
                dir = Vec3::new(1.0, 0.0, 0.0);
            } else {
                dir = dir.normalized();
            }

            // Create per-galaxy template variants to increase morphological diversity with higher counts.
            let mut t1 = accretion_template.clone();
            let mut t2 = accretion_template.clone();
            // Property variance around 1.0 is controlled by galaxy_property_variance_factor.
            // Use (property_variance_scale - 1.0) so a neutral config (scale == 1.0) produces no randomness.
            let amp1 = (fastrand::f32() - 0.5) * 2.0 * ((property_variance_scale - 1.0) * 0.5);
            let amp2 = (fastrand::f32() - 0.5) * 2.0 * ((property_variance_scale - 1.0) * 0.5);
            let mult1 = (1.0 + amp1).clamp(0.25, 4.0);
            let mult2 = (1.0 + amp2).clamp(0.25, 4.0);
            t1.n = ((accretion_template.n as f32) * mult1).round() as usize;
            t2.n = ((accretion_template.n as f32) * mult2).round() as usize;
            // Allocate per-galaxy disk particle counts from the remaining global budget.
            let orig_t1_n = t1.n;
            if remaining_disk_budget >= 0 {
                let fair_share = if remaining_galaxy_slots > 0 && remaining_disk_budget > 0 {
                    (remaining_disk_budget as f32 / remaining_galaxy_slots as f32).floor() as usize
                } else {
                    0usize
                };
                let alloc = orig_t1_n.min(fair_share);
                t1.n = alloc;
                if t1.n != orig_t1_n {
                    eprintln!("⚠️ Galaxy disk particle count capped by total_body_budget: requested {} -> allocated {}", orig_t1_n, t1.n);
                }
                remaining_disk_budget -= t1.n as isize;
                remaining_galaxy_slots = remaining_galaxy_slots.saturating_sub(1);
            } else {
                // no global budget constraint
                remaining_galaxy_slots = remaining_galaxy_slots.saturating_sub(1);
            }

            let orig_t2_n = t2.n;
            if remaining_disk_budget >= 0 {
                let fair_share = if remaining_galaxy_slots > 0 && remaining_disk_budget > 0 {
                    (remaining_disk_budget as f32 / remaining_galaxy_slots as f32).floor() as usize
                } else {
                    0usize
                };
                let alloc = orig_t2_n.min(fair_share);
                t2.n = alloc;
                if t2.n != orig_t2_n {
                    eprintln!("⚠️ Galaxy disk particle count capped by total_body_budget: requested {} -> allocated {}", orig_t2_n, t2.n);
                }
                remaining_disk_budget -= t2.n as isize;
                remaining_galaxy_slots = remaining_galaxy_slots.saturating_sub(1);
            } else {
                remaining_galaxy_slots = remaining_galaxy_slots.saturating_sub(1);
            }
            t1.outer_radius = (accretion_template.outer_radius * mult1.sqrt()).max(2.0);
            t2.outer_radius = (accretion_template.outer_radius * mult2.sqrt()).max(2.0);
            t1.inner_radius = (accretion_template.inner_radius * mult1.sqrt()).max(1.0);
            t2.inner_radius = (accretion_template.inner_radius * mult2.sqrt()).max(1.0);
            t1.central_mass = accretion_template.central_mass * (1.0 + (fastrand::f32() - 0.5) * property_variance_scale * 0.5);
            t2.central_mass = accretion_template.central_mass * (1.0 + (fastrand::f32() - 0.5) * property_variance_scale * 0.5);
            let pmr1 = (
                accretion_template.particle_mass_range.0 * (1.0 + amp1 * 0.4),
                accretion_template.particle_mass_range.1 * (1.0 + amp1 * 0.4),
            );
            let pmr2 = (
                accretion_template.particle_mass_range.0 * (1.0 + amp2 * 0.4),
                accretion_template.particle_mass_range.1 * (1.0 + amp2 * 0.4),
            );
            t1.particle_mass_range = pmr1;
            t2.particle_mass_range = pmr2;
            t1.spin_speed_multiplier = accretion_template.spin_speed_multiplier * (1.0 + (fastrand::f32() - 0.5) * 0.5 * property_variance_scale);
            t2.spin_speed_multiplier = accretion_template.spin_speed_multiplier * (1.0 + (fastrand::f32() - 0.5) * 0.5 * property_variance_scale);


            // Estimate per-galaxy masses using the per-template averages (used for mass-weighted separation)
            let m1_est = t1.central_mass + t1.n as f32 * ((t1.particle_mass_range.0 + t1.particle_mass_range.1) * 0.5);
            let m2_est = t2.central_mass + t2.n as f32 * ((t2.particle_mass_range.0 + t2.particle_mass_range.1) * 0.5);

            // separation can be mass-weighted if enabled
            let mut pair_separation = accretion_template.outer_radius * config.galaxy_separation_factor;
            if config.galaxy_mass_weighted_separation {
                let ratio = ((m1_est + m2_est) / (mass_ref * 2.0)).max(0.25);
                let scale = ratio.powf(config.galaxy_separation_mass_exponent);
                pair_separation *= scale.clamp(0.5, 8.0);
            }

            let center1 = pair_center + dir * (pair_separation * 0.5);
            let center2 = pair_center - dir * (pair_separation * 0.5);

            // random inclinations and rotation sense for each galaxy
            let axis1 = Self::random_inclination_axis();
            let axis2 = Self::random_inclination_axis();
            let clockwise1 = fastrand::bool();
            let clockwise2 = fastrand::bool();

            // generate both galaxy bodies (XY center passed to generator; we'll add Z offset)
            let mut g1 = t1.generate_inclined(Vec2::new(center1.x, center1.y), Vec2::zero(), axis1, clockwise1);
            for b in &mut g1 {
                b.pos.z += center1.z;
            }

            let mut g2 = t2.generate_inclined(Vec2::new(center2.x, center2.y), Vec2::zero(), axis2, clockwise2);
            for b in &mut g2 {
                b.pos.z += center2.z;
            }

            // Elemental pairing and velocity remapping per galaxy when enabled
            if !atomic_numbers.is_empty() {
                Self::assign_element_pair(&mut g1, &atomic_numbers, &core_candidates);
                Self::assign_element_pair(&mut g2, &atomic_numbers, &core_candidates);
                Self::remap_elemental_orbit_velocities(&mut g1, &config);
                Self::remap_elemental_orbit_velocities(&mut g2, &config);
            }

            // compute actual masses and choose bulk velocities (optionally prefer circular binaries)
            let m1: f32 = g1.iter().map(|b| b.mass).sum();
            let m2: f32 = g2.iter().map(|b| b.mass).sum();
            let center1_pos = g1[0].pos;
            let center2_pos = g2[0].pos;
            let (v1, v2) = if fastrand::f32() < config.galaxy_binary_circular_fraction {
                // Near-circular relative velocity for mutual orbit around pair center
                let dist = (center2_pos - center1_pos).mag().max(f32::EPSILON);
                let v_circ = ((m1 + m2) / dist).sqrt();
                let dir_rel = (center2_pos - center1_pos).normalized();
                let reference = if dir_rel.abs().x < 0.9 { Vec3::new(1.0, 0.0, 0.0) } else { Vec3::new(0.0, 1.0, 0.0) };
                let mut perp = dir_rel.cross(reference);
                if perp.mag_sq() <= 1e-8 { perp = Vec3::new(-dir_rel.y, dir_rel.x, 0.0); }
                let perp = perp.normalized();
                let sign = if fastrand::bool() { 1.0 } else { -1.0 };
                let v_rel = perp * v_circ * sign;
                let v1 = v_rel * (m2 / (m1 + m2));
                let v2 = -v_rel * (m1 / (m1 + m2));
                (v1, v2)
            } else {
                Self::galaxy_bulk_velocities(&config, m1, m2, center1_pos, center2_pos)
            };

            for b in &mut g1 { b.vel += v1; }
            for b in &mut g2 { b.vel += v2; }

            // cluster-level bulk motion added on top of pair velocities
            // Use precomputed, coherent cluster initial velocities so members of the same cluster
            // move naturally together. Add a small per-pair jitter to avoid perfectly rigid motion.
            let cluster_vel = if cluster_enabled {
                let base = cluster_init_vels[cid];
                let jitter = Vec3::new(
                    (fastrand::f32() - 0.5) * 0.02,
                    (fastrand::f32() - 0.5) * 0.02,
                    (fastrand::f32() - 0.5) * 0.01,
                ) * scatter_scale_max * 0.001;
                base + jitter
            } else {
                Vec3::zero()
            };
            for b in &mut g1 { b.vel += cluster_vel; }
            for b in &mut g2 { b.vel += cluster_vel; }

            // append bodies and record start indices for cache/slicing
            galaxy_start_indices.push(bodies.len());
            // record cluster id for each body being appended so runtime can apply coherent cluster motion
            for _ in 0..g1.len() { body_cluster_ids.push(cid); }
            bodies.extend(g1);
            galaxy_start_indices.push(bodies.len());
            for _ in 0..g2.len() { body_cluster_ids.push(cid); }
            bodies.extend(g2);
        }

        if config.galaxy_log_centers_on_init {
            eprintln!(
                "🛰️ Galaxy init: clusters={} pairs={} sample_pair_centers={:?}",
                cluster_count,
                pair_count,
                pair_centers_preview
            );
            eprintln!(
                "🛰️ Cluster centers ({}): {:?}",
                cluster_centers.len().min(8),
                cluster_centers.iter().take(8).collect::<Vec<_>>()
            );
        }

        // Ensure we have sensible center indices for legacy code paths.
        let center1_idx = *galaxy_start_indices.get(0).unwrap_or(&0);
        let center2_idx = *galaxy_start_indices.get(1).unwrap_or(&usize::MAX);
        let next_body_id = Self::assign_body_ids(&mut bodies, 1);
        let mut sim = Self {
            dt,
            frame: 0,
            bodies,
            octree: Octree::new(theta, epsilon),
            accretion_template,
            config,
            center1_idx,
            center2_idx,
            equatorial_plane_normal: Vec3::new(0.0, 0.0, 1.0),
            equatorial_plane_center: Vec3::zero(),
            // Persist the per-cluster motion components computed at initialization
            cluster_base_translations,
            cluster_orbit_bases,
            cluster_phases,
            cluster_freqs,
            // Start the dynamic cluster velocity to the initial sample for immediate effect
            cluster_velocities: cluster_init_vels,
            // Mapping of body index -> cluster id for runtime updates
            body_cluster_ids,
            next_body_id,
            next_molecule_id: 1,
            molecular_bonds: Vec::new(),
            timescale_scheduler: TimescaleScheduler::new(),
            galaxies: Vec::new(),
            galaxy_start_indices,
        };

        // Cache per-galaxy snapshots for emergence heuristics and other GPDM workflows
        sim.cache_galaxies();

        sim
    }

    fn assign_body_ids(bodies: &mut [Body], first_id: u64) -> u64 {
        let mut next_id = first_id;
        for body in bodies {
            body.body_id = next_id;
            next_id = next_id.saturating_add(1);
        }
        next_id
    }

    /// Insert a body while keeping body_cluster_ids synchronized.
    /// Returns the index at which the body was inserted.
    fn add_body_with_cluster(&mut self, mut body: Body, cid: Option<usize>) -> usize {
        body.body_id = self.next_body_id;
        self.next_body_id = self.next_body_id.saturating_add(1);
        let assigned_cid = cid.unwrap_or(0usize);
        self.body_cluster_ids.push(assigned_cid);
        self.bodies.push(body);
        self.bodies.len() - 1
    }

    /// Remove the body at `idx` and keep body_cluster_ids and center indices consistent.
    /// Returns the removed body id if any.
    fn remove_body_at(&mut self, idx: usize) -> Option<u64> {
        if idx >= self.bodies.len() {
            return None;
        }
        let removed_id = self.bodies[idx].body_id;
        self.bodies.remove(idx);
        if idx < self.body_cluster_ids.len() {
            self.body_cluster_ids.remove(idx);
        }
        // Adjust center indices if they point past the removed index
        if self.center1_idx == idx {
            self.center1_idx = 0;
        } else if self.center1_idx > idx {
            self.center1_idx = self.center1_idx.saturating_sub(1);
        }
        if self.center2_idx == idx {
            self.center2_idx = usize::MAX;
        } else if self.center2_idx > idx {
            self.center2_idx = self.center2_idx.saturating_sub(1);
        }
        Some(removed_id)
    }

    /// Build cached per-galaxy snapshots from the current `bodies` vector.
    pub fn cache_galaxies(&mut self) {
        self.galaxies.clear();
        if self.bodies.is_empty() {
            return;
        }

        // If we have explicit per-galaxy start indices recorded at initialization, use them
        // to slice the master `bodies` vector into GalaxyState snapshots. Otherwise fall back
        // to the legacy two-galaxy heuristic.
        if !self.galaxy_start_indices.is_empty() {
            let mut gid: u64 = 1;
            for (i, &start) in self.galaxy_start_indices.iter().enumerate() {
                let end = if i + 1 < self.galaxy_start_indices.len() {
                    self.galaxy_start_indices[i + 1]
                } else {
                    self.bodies.len()
                };
                if start >= end || start >= self.bodies.len() {
                    continue;
                }
                let slice = self.bodies[start..end].to_vec();
                self.galaxies.push(GalaxyState::new(gid, slice));
                gid += 1;
            }
            return;
        }

        // Legacy behavior: split into two galaxies if a second center index exists, otherwise
        // treat the full body list as a single galaxy snapshot.
        if self.center2_idx < self.bodies.len() && self.config.galaxy_count >= 2 {
            let mid = self.center2_idx.min(self.bodies.len());
            let first = self.bodies[..mid].to_vec();
            let second = self.bodies[mid..].to_vec();
            self.galaxies.push(GalaxyState::new(1, first));
            self.galaxies.push(GalaxyState::new(2, second));
        } else {
            // Single galaxy snapshot
            self.galaxies.push(GalaxyState::new(1, self.bodies.clone()));
        }
    }

    pub fn recompute_institutions(&mut self) {
        use std::collections::HashSet;
        let mut produced_ids: HashSet<u64> = HashSet::new();
        let mut produced_tables: Vec<crate::tables::Table> = Vec::new();

        for g in &self.galaxies {
            let start_id = crate::id_alloc::next_id();
            let new_tables = crate::emergence::emergence_v1(g, &self.config, start_id);
            for t in new_tables.into_iter() {
                produced_ids.insert(t.id);
                produced_tables.push(t);
            }
        }

        let mut reg = crate::tables::TABLES.lock();
        // Upsert produced tables
        for t in produced_tables.into_iter() {
            if let Some(idx) = reg.iter().position(|r| r.id == t.id) {
                reg[idx] = t;
            } else {
                reg.push(t);
            }
        }
        // Retain only tables produced in this recompute to avoid stale entries
        if !produced_ids.is_empty() {
            reg.retain(|r| produced_ids.contains(&r.id));
        }

        eprintln!(
            "🟦 recompute_institutions: produced {} tables",
            produced_ids.len()
        );
    }

    // Apply a gentle, coherent per-cluster motion field each frame. Cluster velocities are
    // derived from a small base translation and an orbital base vector computed at init;
    // they evolve slowly in time (low-frequency) and are applied as a small acceleration
    // to members of the cluster so movement looks natural without destabilizing the system.
    fn apply_cluster_motion(&mut self) {
        if self.cluster_velocities.is_empty() || self.body_cluster_ids.is_empty() {
            return;
        }
        let t = self.frame as f32;
        // Evolve per-cluster velocities smoothly
        let blend = 0.02_f32; // smoothing factor toward desired
        for cid in 0..self.cluster_velocities.len() {
            let phase = self.cluster_phases[cid];
            let freq = self.cluster_freqs[cid];
            let wobble = (t * freq + phase).sin();
            let desired = (self.cluster_base_translations[cid] + self.cluster_orbit_bases[cid] * wobble)
                * self.config.galaxy_cluster_motion_factor;
            self.cluster_velocities[cid] = self.cluster_velocities[cid] * (1.0 - blend) + desired * blend;
        }

        // Apply a small acceleration proportional to the current cluster velocity to each body
        // in the corresponding cluster. Keep the influence very small to avoid destabilizing
        // fine-grained internal orbits inside galaxies.
        let influence = 0.003_f32;
        let dt = self.dt.max(1e-6);
        for (idx, body) in self.bodies.iter_mut().enumerate() {
            let cid = *self.body_cluster_ids.get(idx).unwrap_or(&0);
            if cid >= self.cluster_velocities.len() { continue; }
            let add_acc = self.cluster_velocities[cid] * (influence / dt);
            body.acc += add_acc;
        }
    }

    fn random_inclination_axis() -> Vec3 {
        let theta = fastrand::f32() * std::f32::consts::PI * 0.5;
        let phi = fastrand::f32() * std::f32::consts::TAU;
        let x = theta.sin() * phi.cos();
        let y = theta.sin() * phi.sin();
        let z = theta.cos();
        Vec3::new(x, y, z).normalized()
    }

    fn new_galactic_atom_simulation(
        source_config: &InformationsConfig,
        accretion_template: GalaxyTemplate,
        dt: f32,
        theta: f32,
        epsilon: f32,
    ) -> Self {
        let config = source_config.clone();
        let mut atomic_numbers = if config.element_atomic_numbers.is_empty() {
            (1..=config.element_max_atomic_number).collect::<Vec<_>>()
        } else {
            config.element_atomic_numbers.clone()
        };
        atomic_numbers.retain(|atomic_number| (1..=118).contains(atomic_number));
        atomic_numbers.sort_unstable();
        atomic_numbers.dedup();
        if atomic_numbers.is_empty() {
            atomic_numbers.push(1);
        }

        let total_body_budget = config.element_atomic_count.max(atomic_numbers.len());
        let base_body_count = total_body_budget / atomic_numbers.len();
        let remainder = total_body_budget % atomic_numbers.len();
        let grid_width = (atomic_numbers.len() as f32).sqrt().ceil() as usize;
        let spacing = config.outer_radius * config.element_universe_scatter_factor.max(1.0) * 2.2;
        let grid_center = (grid_width.saturating_sub(1) as f32) * spacing * 0.5;
        let mut bodies = Vec::with_capacity(total_body_budget.max(atomic_numbers.len()));

        for (index, atomic_number) in atomic_numbers.iter().copied().enumerate() {
            let Some(element) = element_by_atomic_number(atomic_number) else {
                continue;
            };
            let body_count = (base_body_count + usize::from(index < remainder))
                .max(if atomic_number == 1 { 400 } else { 1 });
            let radius_ratio = (element.covalent_radius_pm / 100.0).clamp(0.4, 2.5);
            let dimensional_scale = (config.element_radius_scale / 5.0).clamp(0.1, 10.0);
            let outer_radius =
                (accretion_template.outer_radius * radius_ratio * dimensional_scale).max(2.0);
            let inner_radius = (accretion_template.inner_radius * radius_ratio * dimensional_scale)
                .clamp(1.0, outer_radius * 0.45);
            let atomic_mass_ratio = (element.atomic_weight / 1.008).max(0.01);
            let central_mass =
                config.central_mass * config.element_center_mass_unit.max(0.0) * atomic_mass_ratio;
            let mass_scale = config.element_orbital_mass_unit.max(0.0) * atomic_mass_ratio;
            let particle_mass_range = (
                config.particle_mass_range.0 * mass_scale,
                config.particle_mass_range.1 * mass_scale,
            );
            let template = GalaxyTemplate {
                n: body_count.saturating_sub(1),
                inner_radius,
                outer_radius,
                disk_equilibrium_enabled: config.enable_disk_equilibrium_mode,
                disk_equilibrium_softening: config.epsilon * config.softening_scale_factor,
                disk_equilibrium_scale_height_factor: config.disk_equilibrium_scale_height_factor,
                disk_equilibrium_asymmetric_drift_strength: config
                    .disk_equilibrium_asymmetric_drift_strength,
                central_mass,
                particle_mass_range,
                spin_speed_multiplier: config.spin_speed_multiplier,
                accretion_spawn_rate: config.accretion_spawn_rate,
                outer_ring_spawn_zone_inner_radius: outer_radius * 0.82,
                outer_ring_spawn_zone_outer_radius: outer_radius,
            };
            let grid_x = index % grid_width;
            let grid_y = index / grid_width;
            let center = Vec2::new(
                grid_x as f32 * spacing - grid_center,
                grid_y as f32 * spacing - grid_center,
            );
            let axis = Self::random_inclination_axis();
            let clockwise = fastrand::bool();
            let mut system_bodies =
                template.generate_inclined(center, Vec2::zero(), axis, clockwise);
            let gas_start_index = system_bodies.len() * 9 / 10;

            for (body_index, body) in system_bodies.iter_mut().enumerate() {
                let distance = (body.pos - Vec3::new(center.x, center.y, 0.0)).mag();
                let segment = if body_index == 0 {
                    BodySegment::Core
                } else if body_index >= gas_start_index {
                    BodySegment::Gas
                } else if distance < inner_radius + (outer_radius - inner_radius) * 0.22 {
                    BodySegment::Bulge
                } else {
                    BodySegment::Orbital
                };
                *body = body.with_element(atomic_number, segment);
                if segment == BodySegment::Gas {
                    *body = body.with_gas_state(
                        (body.mass / central_mass.max(1.0)).clamp(0.01, 1.0),
                        body.vel.mag().clamp(0.0, 100.0),
                    );
                }
            }
            Self::remap_elemental_orbit_velocities(&mut system_bodies, &config);
            bodies.append(&mut system_bodies);
        }

        let equatorial_plane_center = bodies
            .first()
            .map(|body| body.pos)
            .unwrap_or_else(Vec3::zero);
        let mut config_template = accretion_template;
        config_template.n = 0;
        let next_body_id = Self::assign_body_ids(&mut bodies, 1);
        let body_count = bodies.len();

        let mut sim = Self {
            dt,
            frame: 0,
            bodies,
            octree: Octree::new(theta, epsilon),
            accretion_template: config_template,
            config,
            center1_idx: 0,
            center2_idx: usize::MAX,
            equatorial_plane_normal: Vec3::new(0.0, 0.0, 1.0),
            equatorial_plane_center,
            // Cluster motion persisted state - empty for atom mode
            cluster_base_translations: Vec::new(),
            cluster_orbit_bases: Vec::new(),
            cluster_phases: Vec::new(),
            cluster_freqs: Vec::new(),
            cluster_velocities: Vec::new(),
            body_cluster_ids: vec![0usize; body_count],
            next_body_id,
            next_molecule_id: 1,
            molecular_bonds: Vec::new(),
            timescale_scheduler: TimescaleScheduler::new(),
            galaxies: Vec::new(),
            galaxy_start_indices: vec![0],
        };

        sim.cache_galaxies();

        sim
    }

    fn assign_element_pair(bodies: &mut [Body], atomic_numbers: &[u8], core_candidates: &[u8]) {
        if atomic_numbers.is_empty() || bodies.is_empty() {
            return;
        }

        let core_atomic_number = core_candidates
            .iter()
            .filter_map(|atomic_number| element_by_atomic_number(*atomic_number))
            .max_by(|first, second| {
                first
                    .core_suitability()
                    .total_cmp(&second.core_suitability())
            })
            .map(|element| element.atomic_number)
            .unwrap_or(atomic_numbers[0]);
        let gas_start_index = bodies.len() * 9 / 10;

        for (index, body) in bodies.iter_mut().enumerate() {
            if index == 0 {
                *body = body.with_element(core_atomic_number, BodySegment::Core);
            } else {
                let atomic_number = atomic_numbers[(index - 1) % atomic_numbers.len()];
                let segment = if index >= gas_start_index {
                    BodySegment::Gas
                } else {
                    BodySegment::Orbital
                };
                *body = body.with_element(atomic_number, segment);
                if segment == BodySegment::Gas {
                    *body = body.with_gas_state(
                        (body.mass / 10.0).clamp(0.01, 1.0),
                        body.vel.mag().clamp(0.0, 100.0),
                    );
                }
            }
        }
    }

    fn elemental_gravity_scale(body: &Body, config: &InformationsConfig) -> f32 {
        if !config.enable_scientific_element_gravity {
            return 1.0;
        }

        body.element_atomic_number
            .and_then(element_by_atomic_number)
            .map(|element| {
                element.scientific_gravity_scale(
                    config.element_prime_beta,
                    [
                        config.element_gravity_mass_weight,
                        config.element_gravity_electronegativity_weight,
                        config.element_gravity_radius_weight,
                        config.element_gravity_reactivity_weight,
                    ],
                )
            })
            .unwrap_or(1.0)
    }

    fn remap_elemental_orbit_velocities(bodies: &mut [Body], config: &InformationsConfig) {
        if !config.enable_scientific_element_gravity || bodies.is_empty() {
            return;
        }

        let center_position = bodies[0].pos;
        let center_velocity = bodies[0].vel;
        let mut enclosed_mass = 0.0_f32;
        for body in bodies.iter_mut() {
            enclosed_mass += body.mass * Self::elemental_gravity_scale(body, config);
            let offset = body.pos - center_position;
            let radius = offset.mag();
            if radius <= f32::EPSILON {
                continue;
            }

            let tangent = (body.vel - center_velocity).normalized();
            if tangent != Vec3::zero() {
                let circular_speed = if config.enable_disk_equilibrium_mode {
                    plummer_circular_speed(
                        enclosed_mass,
                        radius,
                        config.epsilon * config.softening_scale_factor,
                    )
                } else {
                    (enclosed_mass / radius).sqrt()
                };
                let drift_factor = if config.enable_disk_equilibrium_mode {
                    1.0 - config.disk_equilibrium_asymmetric_drift_strength
                        * (radius / config.outer_radius).clamp(0.0, 1.0)
                        * 0.1
                } else {
                    1.0
                };
                body.vel = center_velocity + tangent * circular_speed * drift_factor;
            }
        }
    }

    pub fn update_roles(&mut self) {
        // Deterministic M1 role update and low-amplitude role effects.
        // This expands the previous simple mapping by computing a broader set
        // of role weights from body state (gas, spin, magnetization, bias)
        // and then applying small, controlled accelerations that map roles to
        // behaviour (inflow, damping, tangential transport, jets/winds, escape).
        if !self.config.enable_flow_dynamics_model {
            return;
        }

        // First pass: compute role weights
        for body in self.bodies.iter_mut() {
            let gas = body.gas_density.max(0.0);
            let spin = body.angular_speed.abs().min(1.0e3);
            let mag = body.magnetization.abs().min(1.0e3);
            let bias = body.accretion_ejection_bias;

            // If no triggers (gas, magnetization, and bias are all effectively zero),
            // keep the default reservoir-dominated roles to match expectations of
            // tests that assert reservoir dominance when nothing is active.
            if gas <= f32::EPSILON && mag <= f32::EPSILON && bias.abs() <= f32::EPSILON {
                body.roles = crate::roles::RoleWeights::default();
                continue;
            }

            // Heuristic raw scores (tunable later)
            let accretion = (gas / (1.0 + gas)) + (bias.max(0.0) * 0.5);
            let densifier = (gas / (2.0 + gas)) * 0.6;
            let transporter = (spin * 0.04).clamp(0.0, 1.0);
            let heater = (body.gas_temperature / (1.0 + body.gas_temperature)) * 0.3;
            let wind = (spin * 0.02).clamp(0.0, 1.0) * bias.max(0.0);
            let jet = (mag * 0.05).clamp(0.0, 1.0);
            let escapee = bias.min(0.0).abs() * 0.8;
            let reservoir: f32 = 0.1; // baseline reservoir to avoid zero-sum

            let mut rw = body.roles;
            rw.accretor = accretion.max(0.0);
            rw.densifier = densifier.max(0.0);
            rw.transporter = transporter.max(0.0);
            rw.heater = heater.max(0.0);
            rw.wind = wind.max(0.0);
            rw.jet = jet.max(0.0);
            rw.escapee = escapee.max(0.0);
            rw.reservoir = reservoir.max(0.0);
            rw.sanitize_and_normalize();
            body.roles = rw;
        }

        // Second pass: apply small role-based accelerations (non-destructive)
        let center = self.equatorial_plane_center;
        for body in self.bodies.iter_mut() {
            let rw = body.roles;
            let mut role_acc = Vec3::zero();

            // Accretor: slight inward pull toward the equatorial center.
            if rw.accretor > 0.0 {
                let dir = center - body.pos;
                let dist = dir.mag().max(1e-6);
                let inward = dir / dist;
                let strength = self.config.inflow_strength * 0.5 * rw.accretor;
                role_acc += inward * strength;
            }

            // Densifier: mild damping (acc opposite to velocity)
            if rw.densifier > 0.0 {
                let damping = self.config.restore_strength * 0.25 * rw.densifier;
                role_acc += -body.vel * damping;
            }

            // Transporter: small tangential nudge to assist redistribution
            if rw.transporter > 0.0 {
                let radial = body.pos - center;
                let radial_mag = radial.mag();
                if radial_mag > f32::EPSILON {
                    let radial_dir = radial / radial_mag;
                    let tangent = Vec3::new(-radial_dir.y, radial_dir.x, 0.0);
                    let t_strength = self.config.inflow_strength * 0.15 * rw.transporter;
                    role_acc += tangent * t_strength;
                }
            }

            // Wind/Jet: along rotation axis (outflow)
            if rw.wind > 0.0 || rw.jet > 0.0 {
                let axis = if body.rotation_axis == Vec3::zero() {
                    Vec3::new(0.0, 0.0, 1.0)
                } else {
                    body.rotation_axis.normalized()
                };
                let wind_strength = self.config.restore_strength * 0.06 * rw.wind;
                let jet_strength = self.config.restore_strength * 0.10 * rw.jet;
                role_acc += axis * (wind_strength + jet_strength);
            }

            // Escapee: outward radial push
            if rw.escapee > 0.0 {
                let radial = body.pos - center;
                let rmag = radial.mag();
                if rmag > f32::EPSILON {
                    let outward = radial / rmag;
                    role_acc += outward * (self.config.inflow_strength * 0.6 * rw.escapee);
                }
            }

            // Heater: tiny stochastic perturbation (models thermal agitation)
            if rw.heater > 0.0 {
                let jitter = Vec3::new(
                    (fastrand::f32() - 0.5) * 0.02,
                    (fastrand::f32() - 0.5) * 0.02,
                    (fastrand::f32() - 0.5) * 0.02,
                );
                role_acc += jitter * rw.heater * 0.5;
            }

            // Safety: only apply finite accelerations
            if role_acc.x.is_finite() && role_acc.y.is_finite() && role_acc.z.is_finite() {
                body.acc += role_acc;
            }
        }
    }

    fn form_molecular_bond(&mut self, first_index: usize, second_index: usize) -> bool {
        if !self.config.enable_molecular_bonding || first_index == second_index {
            return false;
        }

        let first = self.bodies[first_index];
        let second = self.bodies[second_index];
        let (Some(first_number), Some(second_number)) =
            (first.element_atomic_number, second.element_atomic_number)
        else {
            return false;
        };
        let (Some(first_element), Some(second_element)) = (
            element_by_atomic_number(first_number),
            element_by_atomic_number(second_number),
        ) else {
            return false;
        };

        let tendency = binding_tendency(first_element, second_element);
        let same_element_diatomic = first_number == second_number
            && first.segment == second.segment
            && first.segment != BodySegment::Core
            && matches!(first_number, 1 | 7 | 8 | 9 | 17 | 35 | 53);
        if !same_element_diatomic
            && (first_number == second_number || tendency == BindingTendency::Inert)
        {
            return false;
        }

        let displacement = second.pos - first.pos;
        let distance = displacement.mag();
        let contact_distance = first.effective_radius() + second.effective_radius();
        if distance <= f32::EPSILON
            || distance > contact_distance
            || displacement.dot(second.vel - first.vel) >= 0.0
        {
            return false;
        }

        if self.molecular_bonds.iter().any(|bond| {
            (bond.first_body_id == first.body_id && bond.second_body_id == second.body_id)
                || (bond.first_body_id == second.body_id && bond.second_body_id == first.body_id)
        }) {
            return true;
        }

        let first_group_size = molecule_size(&self.bodies, first.molecule_id);
        let second_group_size = molecule_size(&self.bodies, second.molecule_id);
        let combined_size =
            if first.molecule_id.is_some() && first.molecule_id == second.molecule_id {
                first_group_size
            } else {
                first_group_size + second_group_size
            };
        if combined_size > self.config.molecule_max_bodies.max(2) {
            return false;
        }

        self.molecular_bonds.push(MolecularBond {
            first_body_id: first.body_id,
            second_body_id: second.body_id,
            rest_length: (contact_distance * 0.85).max(0.01),
            tendency,
        });
        self.refresh_molecule_memberships();
        true
    }

    pub(crate) fn apply_molecular_bond_forces(&mut self) {
        if !self.config.enable_molecular_bonding || self.molecular_bonds.is_empty() {
            return;
        }

        let body_indices: HashMap<u64, usize> = self
            .bodies
            .iter()
            .enumerate()
            .map(|(index, body)| (body.body_id, index))
            .collect();
        let mut accelerations = vec![Vec3::zero(); self.bodies.len()];
        let mut active_bonds = Vec::with_capacity(self.molecular_bonds.len());
        let mut broke_bond = false;

        for bond in self.molecular_bonds.iter().copied() {
            let (Some(&first_index), Some(&second_index)) = (
                body_indices.get(&bond.first_body_id),
                body_indices.get(&bond.second_body_id),
            ) else {
                broke_bond = true;
                continue;
            };
            let first = self.bodies[first_index];
            let second = self.bodies[second_index];
            let displacement = second.pos - first.pos;
            let distance = displacement.mag();
            if distance <= f32::EPSILON {
                active_bonds.push(bond);
                continue;
            }

            let relative_velocity = second.vel - first.vel;
            let break_speed = self.config.molecule_break_energy_threshold.max(0.0).sqrt()
                * (1.0 + self.config.molecule_bond_strength_factor.max(0.0));
            if relative_velocity.mag() > break_speed {
                broke_bond = true;
                continue;
            }

            let direction = displacement / distance;
            let radial_speed = relative_velocity.dot(direction);
            let tendency_strength = match bond.tendency {
                BindingTendency::Inert => 0.0,
                BindingTendency::Metallic => 0.45,
                BindingTendency::NonPolarCovalent => 0.7,
                BindingTendency::PolarCovalent => 0.85,
                BindingTendency::Ionic => 1.0,
            };
            let stiffness = self.config.molecule_bond_strength_factor.max(0.0) * tendency_strength;
            let damping = stiffness.sqrt() * 0.1;
            let force_magnitude = ((distance - bond.rest_length) * stiffness
                + radial_speed * damping)
                .clamp(-1.0e6, 1.0e6);
            let force = direction * force_magnitude;
            accelerations[first_index] += force / first.mass.max(1.0e-6);
            accelerations[second_index] -= force / second.mass.max(1.0e-6);
            active_bonds.push(bond);
        }

        self.molecular_bonds = active_bonds;
        for (body, acceleration) in self.bodies.iter_mut().zip(accelerations) {
            body.acc += acceleration;
        }
        if broke_bond {
            self.refresh_molecule_memberships();
        }
    }

    fn refresh_molecule_memberships(&mut self) {
        let previous_ids: HashMap<u64, Option<u64>> = self
            .bodies
            .iter()
            .map(|body| (body.body_id, body.molecule_id))
            .collect();
        let mut adjacency = HashMap::<u64, Vec<u64>>::new();
        for bond in &self.molecular_bonds {
            adjacency
                .entry(bond.first_body_id)
                .or_default()
                .push(bond.second_body_id);
            adjacency
                .entry(bond.second_body_id)
                .or_default()
                .push(bond.first_body_id);
        }

        for body in &mut self.bodies {
            body.molecule_id = None;
        }

        let mut visited = HashSet::new();
        let mut used_molecule_ids = HashSet::new();
        let body_ids: Vec<u64> = self.bodies.iter().map(|body| body.body_id).collect();
        for body_id in body_ids {
            if visited.contains(&body_id) || !adjacency.contains_key(&body_id) {
                continue;
            }
            let mut stack = vec![body_id];
            let mut component = Vec::new();
            while let Some(body_id) = stack.pop() {
                if !visited.insert(body_id) {
                    continue;
                }
                component.push(body_id);
                if let Some(neighbors) = adjacency.get(&body_id) {
                    stack.extend(neighbors.iter().copied());
                }
            }
            if component.len() < 2 {
                continue;
            }

            component.sort_unstable();
            let retained_id = component
                .iter()
                .filter_map(|body_id| previous_ids.get(body_id).copied().flatten())
                .find(|molecule_id| !used_molecule_ids.contains(molecule_id));
            let molecule_id = retained_id.unwrap_or_else(|| {
                let molecule_id = self.next_molecule_id;
                self.next_molecule_id = self.next_molecule_id.saturating_add(1);
                molecule_id
            });
            used_molecule_ids.insert(molecule_id);

            for body in &mut self.bodies {
                if component.binary_search(&body.body_id).is_ok() {
                    body.molecule_id = Some(molecule_id);
                }
            }
        }
    }

    pub fn identified_compound(
        &self,
        molecule_id: u64,
    ) -> Option<&'static crate::molecules::CompoundDefinition> {
        if !self.config.molecule_identify_compounds {
            return None;
        }

        let mut composition = HashMap::<u8, u16>::new();
        for body in self
            .bodies
            .iter()
            .filter(|body| body.molecule_id == Some(molecule_id))
        {
            let atomic_number = body.element_atomic_number?;
            let count = composition.entry(atomic_number).or_default();
            *count = count.checked_add(1)?;
        }
        let composition: Vec<_> = composition.into_iter().collect();
        crate::molecules::identify_compound(&composition)
    }
    fn galaxy_bulk_velocities(
        config: &InformationsConfig,
        m1: f32,
        m2: f32,
        center1: Vec3,
        center2: Vec3,
    ) -> (Vec3, Vec3) {
        let distance = (center2 - center1).mag().max(config.outer_radius * 6.0);
        let mu = 1.0;
        let escape_speed = ((2.0 * mu * (m1 + m2)) / distance).sqrt();

        let roll = fastrand::f32();
        let interaction_type = if roll < config.prob_merge {
            0
        } else if roll < config.prob_merge + config.prob_repeated {
            1
        } else {
            2
        };
        let (speed_factor, lateral_factor, angle) = match interaction_type {
            0 => (
                config.merge_speed_factor,
                config.merge_speed_factor * 0.1,
                config.merge_angle,
            ),
            1 => (
                config.repeated_speed_factor,
                config.repeated_speed_factor * 0.2,
                config.repeated_angle,
            ),
            _ => (
                config.flyby_speed_factor,
                config.flyby_speed_factor * 0.25,
                config.flyby_angle,
            ),
        };

        let direction = (center2 - center1).normalized();
        // Build a true perpendicular vector to `direction` in 3D. Prefer an X-reference
        // unless the direction is nearly parallel to X, then use Y. Fallback to the old
        // XY-perp if numerical degeneracy occurs.
        let reference = if direction.abs().x < 0.9 {
            Vec3::new(1.0, 0.0, 0.0)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };
        let mut perp = direction.cross(reference);
        if perp.mag_sq() <= 1e-8 {
            perp = Vec3::new(-direction.y, direction.x, 0.0);
        }
        let perp = perp.normalized();
        let sign = if fastrand::bool() { 1.0 } else { -1.0 };

        let v_radial = direction * escape_speed * speed_factor;
        let v_tangent = perp * escape_speed * lateral_factor * sign;
        let v_rel = v_radial * angle.cos() + v_tangent * angle.sin();

        let v1 = v_rel * (m2 / (m1 + m2));
        let v2 = -v_rel * (m1 / (m1 + m2));

        (v1, v2)
    }

    pub fn step(&mut self) {
        // Advance the timescale scheduler once per simulation step
        self.timescale_scheduler.tick();

        // Role updates (M1) are low-frequency and governed by role_update_interval
        if self
            .timescale_scheduler
            .should_update(self.config.role_update_interval)
        {
            if self.config.enable_flow_dynamics_model {
                self.update_roles();
                eprintln!("🧭 Role update executed at frame {}", self.frame);
            }
        }

        self.process_manual_spawns();
        self.iterate();
        if self.frame % self.config.collision_interval == 0 {
            if renderer::COLLISIONS_ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
                self.collide();
            }
        }
        if self.frame % self.config.attract_interval == 0 {
            self.attract();
            // Raumzeitkrümmungs-Dilatation für Center-Partikel überschreibt Teil der Gravitation
            self.apply_center_spacetime_dilation();
        }
        if renderer::SPAWN_ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
            self.spawn_accretion();
        }
        if self.config.enable_flow_dynamics_model {
            // deterministically detect dense gas cells and spawn proto-stars conservatively
            self.process_gas_collapse_and_spawn();
        }
        // Periodically compute emergence heuristics to instantiate institutions (M3+ behavior)
        if self
            .timescale_scheduler
            .should_update(self.config.institution_recompute_interval)
        {
            if self.config.enable_flow_dynamics_model {
                // Refresh galaxy snapshots and run emergence recompute to instantiate institutions (M3+)
                self.cache_galaxies();
                self.recompute_institutions();
            }
        }

        self.apply_cluster_motion();
        if self.config.enable_flow_dynamics_model {
            self.update_star_evolution();
        }
        self.frame += 1;
    }

    pub fn render_update_interval(&self) -> usize {
        self.config.render_update_interval.max(1)
    }

    pub fn attract(&mut self) {
        let oct = Oct::new_containing(&self.bodies);
        // Allow tuning the octree reserve size via configuration to balance memory vs realloc
        let reserve_factor = self.config.performance_octree_reserve_factor.max(1.0);
        let reserve_nodes = ((self.bodies.len() as f32) * reserve_factor).ceil() as usize;
        // parents typically fewer than leaf nodes; reserve a fraction of the factor
        let reserve_parents = ((self.bodies.len() as f32) * (reserve_factor * 0.25)).ceil() as usize;
        self.octree.reserve(reserve_nodes, reserve_parents);
        self.octree.clear(oct);

        for body in &self.bodies {
            let effective_mass = body.mass * Self::elemental_gravity_scale(body, &self.config);
            self.octree.insert(body.pos, effective_mass);
        }

        self.octree.propagate();

        let softening = self.config.epsilon * self.config.softening_scale_factor;
        let softening_sq = softening * softening;
        self.bodies.par_iter_mut().for_each(|body| {
            body.acc = self.octree.acc(body.pos, softening_sq);
        });

        self.apply_hydrodynamic_inflow();
        self.apply_molecular_bond_forces();
    }

    fn apply_hydrodynamic_inflow(&mut self) {
        let outer_r = self.accretion_template.outer_radius.max(1.0);
        let inflow_strength = self.config.inflow_strength;
        let restore_strength = self.config.restore_strength;
        let plane_center = self.equatorial_plane_center;
        let plane_normal = self.equatorial_plane_normal;

        self.bodies.par_iter_mut().for_each(|body| {
            let offset = body.pos - plane_center;
            let dist_to_plane = offset.dot(plane_normal);
            let in_plane_pos = offset - plane_normal * dist_to_plane;
            let in_plane_dist = in_plane_pos.mag();

            let plane_attraction = -plane_normal * dist_to_plane * restore_strength / outer_r;
            let radial_inflow = if in_plane_dist > 0.0 {
                -in_plane_pos / in_plane_dist
                    * inflow_strength
                    * (1.0 - (in_plane_dist / outer_r).clamp(0.0, 1.0))
            } else {
                Vec3::zero()
            };

            body.acc += plane_attraction + radial_inflow;
        });
    }

    /// Raumzeitkrümmungs-Dilatation für Center-zu-Center Anziehung mit Spiralisierungs-Effekt
    /// Die Zentren nähern sich während ihrer Umkreisung an (orbitale Energie-Dissipation)
    fn apply_center_spacetime_dilation(&mut self) {
        if self.center1_idx >= self.bodies.len() || self.center2_idx >= self.bodies.len() {
            return;
        }

        let center1_pos = self.bodies[self.center1_idx].pos;
        let center1_vel = self.bodies[self.center1_idx].vel;
        let center1_mass = self.bodies[self.center1_idx].mass;
        let center2_pos = self.bodies[self.center2_idx].pos;
        let center2_vel = self.bodies[self.center2_idx].vel;
        let center2_mass = self.bodies[self.center2_idx].mass;

        let delta = center2_pos - center1_pos;
        let distance = delta.mag();

        if distance < 1e-6 {
            return; // Zu nah, ignoriere
        }

        let direction = delta.normalized();

        // 1. GRAVITATIONS-KRAFT (anziehendes Potential)
        // Stärke erhöht um Annäherung zu forcieren
        let force_magnitude = (center1_mass * center2_mass) / (distance * distance);
        let grav_force = direction * force_magnitude;

        // 2. DISSIPATIONS-KRAFT (Gravitationswellenstrahlung simulieren)
        // Entzieht dem System Energie basierend auf Relativgeschwindigkeit und Abstand
        let rel_vel = center2_vel - center1_vel;
        let radial_vel = rel_vel.dot(direction); // Nur die radiale Komponente

        // Dissipation ist stark, wenn Relative-Geschwindigkeit hoch ist und Abstand klein ist
        // Dies führt zu einer Spirale ins Zentrum
        let dissipation_factor = self.config.spacetime_dilation_factor * 0.001; // Bremsfaktor
        let dissipation_force =
            -direction * (radial_vel.abs() * force_magnitude * dissipation_factor);

        // 3. GESAMTKRAFT = Gravitation + Dissipation
        let total_force = grav_force + dissipation_force;

        // Wende Kraft auf beide Center an (in entgegengesetzter Richtung)
        if self.center1_idx < self.bodies.len() {
            self.bodies[self.center1_idx].acc += total_force / center1_mass.max(1.0);
        }
        if self.center2_idx < self.bodies.len() {
            self.bodies[self.center2_idx].acc -= total_force / center2_mass.max(1.0);
        }

        eprintln!(
            "🌌 Spiral: dist={:.2}, grav={:.2}, dissipation={:.2}",
            distance,
            force_magnitude,
            dissipation_force.mag()
        );
    }

    pub fn iterate(&mut self) {
        self.bodies.par_iter_mut().for_each(|body| {
            body.update(self.dt);
        });
    }

    /// Profiled step: execute a simulation step and return per-component timings
    /// as (name, duration_micros). Currently returns a single `step_total` component.
    pub fn step_profiled(&mut self) -> Vec<(String, u128)> {
        let t0 = std::time::Instant::now();
        self.step();
        let dur = t0.elapsed().as_micros() as u128;
        vec![("step_total".to_string(), dur)]
    }

    pub fn collide(&mut self) {
        if self.bodies.len() < 2 {
            return;
        }

        let mut center_merge_pair: Option<(usize, usize)> = None;

        let max_radius = self
            .bodies
            .iter()
            .map(|body| body.effective_radius())
            .fold(0.0_f32, f32::max)
            .max(1.0);
        let mut cell_size = max_radius * 3.0;

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut min_z = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut max_z = f32::MIN;
        for body in &self.bodies {
            min_x = min_x.min(body.pos.x);
            min_y = min_y.min(body.pos.y);
            min_z = min_z.min(body.pos.z);
            max_x = max_x.max(body.pos.x);
            max_y = max_y.max(body.pos.y);
            max_z = max_z.max(body.pos.z);
        }

        min_x -= cell_size;
        min_y -= cell_size;
        min_z -= cell_size;
        max_x += cell_size;
        max_y += cell_size;
        max_z += cell_size;

        let mut size_x = ((max_x - min_x) / cell_size).ceil() as usize + 1;
        let mut size_y = ((max_y - min_y) / cell_size).ceil() as usize + 1;
        let mut size_z = ((max_z - min_z) / cell_size).ceil() as usize + 1;
        let max_grid_dim = 128_usize;
        let max_total_cells = max_grid_dim
            .saturating_mul(max_grid_dim)
            .saturating_mul(max_grid_dim);

        if size_x > max_grid_dim || size_y > max_grid_dim || size_z > max_grid_dim {
            let scale = ((size_x.max(size_y).max(size_z)) as f32 / max_grid_dim as f32).ceil();
            cell_size *= scale;
            size_x = ((max_x - min_x) / cell_size).ceil() as usize + 1;
            size_y = ((max_y - min_y) / cell_size).ceil() as usize + 1;
            size_z = ((max_z - min_z) / cell_size).ceil() as usize + 1;
        }

        let total_cells = size_x.saturating_mul(size_y).saturating_mul(size_z);
        if total_cells == 0 || total_cells > max_total_cells {
            return;
        }

        // Build a sparse map of occupied grid cells to avoid large empty allocations
        let cell_index = |pos: Vec3| {
            let ix = ((pos.x - min_x) / cell_size).floor() as isize;
            let iy = ((pos.y - min_y) / cell_size).floor() as isize;
            let iz = ((pos.z - min_z) / cell_size).floor() as isize;
            ((ix as usize) * size_y + iy as usize) * size_z + iz as usize
        };

        let mut cell_map: HashMap<usize, Vec<usize>> = HashMap::new();
        for (index, body) in self.bodies.iter().enumerate() {
            let idx = cell_index(body.pos);
            if idx < total_cells {
                cell_map.entry(idx).or_default().push(index);
            }
        }

        if cell_map.is_empty() {
            return;
        }

        let cell_map = Arc::new(cell_map);
        let occupied_cells: Vec<usize> = cell_map.as_ref().keys().copied().collect();

        // Collect candidate pairs in parallel (no locking), then resolve sequentially to avoid data races
        let pairs_by_cell: Vec<Vec<(usize, usize)>> = occupied_cells
            .par_iter()
            .map(|&cell_idx| {
                let map_ref = &*cell_map;
                let size_y_sz = size_y;
                let size_z_sz = size_z;
                let size_x_sz = size_x;

                let ix = cell_idx / (size_y_sz * size_z_sz);
                let rem = cell_idx % (size_y_sz * size_z_sz);
                let iy = rem / size_z_sz;
                let iz = rem % size_z_sz;

                let mut local_pairs: Vec<(usize, usize)> = Vec::new();
                if let Some(indices) = map_ref.get(&cell_idx) {
                    for dx in 0..=1 {
                        for dy in 0..=1 {
                            for dz in 0..=1 {
                                let nx = ix + dx;
                                let ny = iy + dy;
                                let nz = iz + dz;
                                if nx >= size_x_sz || ny >= size_y_sz || nz >= size_z_sz {
                                    continue;
                                }
                                let neighbor_idx = (nx * size_y_sz + ny) * size_z_sz + nz;
                                if let Some(neighbor_indices) = map_ref.get(&neighbor_idx) {
                                    for &i in indices {
                                        for &j in neighbor_indices {
                                            if neighbor_idx == cell_idx && i >= j {
                                                continue;
                                            }
                                            if i == j {
                                                continue;
                                            }
                                            local_pairs.push((i, j));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                local_pairs
            })
            .collect();

        let index_pairs: Vec<(usize, usize)> = pairs_by_cell.into_iter().flatten().collect();

        // Convert index pairs to stable id pairs to avoid index invalidation if bodies are reordered later
        let mut id_pairs: Vec<(u64, u64)> = Vec::with_capacity(index_pairs.len());
        for (i, j) in index_pairs {
            if i < self.bodies.len() && j < self.bodies.len() {
                id_pairs.push((self.bodies[i].body_id, self.bodies[j].body_id));
            }
        }

        // Map current body_id -> index for stable lookups during sequential resolution
        let id_index_map: HashMap<u64, usize> = self
            .bodies
            .iter()
            .enumerate()
            .map(|(idx, body)| (body.body_id, idx))
            .collect();

        for (id1, id2) in id_pairs {
            if let (Some(&i), Some(&j)) = (id_index_map.get(&id1), id_index_map.get(&id2)) {
                self.resolve(i, j, &mut center_merge_pair);
            }
        }

        if let Some((i, j)) = center_merge_pair {
            self.merge_center_particles(i, j);
        }
    }

    fn element_numbers_for_identity(&self) -> Vec<u8> {
        if self.config.enable_galactic_atom_simulation {
            if self.config.element_atomic_numbers.is_empty() {
                (1..=self.config.element_max_atomic_number).collect()
            } else {
                self.config.element_atomic_numbers.clone()
            }
        } else if self.config.enable_elemental_galaxy_pair_mode && self.config.n > 200 {
            if self.config.element_pair_atomic_numbers.is_empty() {
                all_elements()
                    .map(|element| element.atomic_number)
                    .collect()
            } else {
                self.config.element_pair_atomic_numbers.clone()
            }
        } else {
            Vec::new()
        }
    }

    /// Spawns accretion particles in the outer ring spawn zone.
    ///
    /// On each step rolls against `accretion_spawn_rate`. A successful roll places
    /// one new particle at a random position within the outer ring spawn zone
    /// (outer third of the galactic disk) and assigns it a circular orbital velocity
    /// derived from the local gravitational acceleration reported by the octree.
    /// Spawns are placed in absolute disk coordinates, avoiding the central bulge.
    fn spawn_accretion(&mut self) {
        let spawn_start_r = self.accretion_template.outer_ring_spawn_zone_inner_radius;
        let outer_r = self.accretion_template.outer_ring_spawn_zone_outer_radius;
        if spawn_start_r >= outer_r {
            return;
        }

        if fastrand::f32() > self.accretion_template.accretion_spawn_rate {
            return;
        }

        let element_numbers = self.element_numbers_for_identity();
        let atomic_number = if element_numbers.is_empty() {
            None
        } else {
            Some(element_numbers[fastrand::usize(0..element_numbers.len())])
        };
        let (mass_min, mass_max) = self.accretion_template.particle_mass_range;

        // Determine spawn center body and its rotation axis + direction, prefer element-specific core in atom mode.
        let spawn_center_idx_opt = if self.config.enable_galactic_atom_simulation {
            atomic_number.and_then(|selected| {
                self.bodies.iter().position(|body| {
                    body.element_atomic_number == Some(selected) && body.segment == BodySegment::Core
                })
            })
        } else {
            Some(self.center1_idx)
        };

        let spawn_center_idx = spawn_center_idx_opt.unwrap_or(self.center1_idx);
        let spawn_center_body = self.bodies.get(spawn_center_idx);
        let spawn_center = spawn_center_body.map(|b| b.pos).unwrap_or_else(Vec3::zero);
        let rotation_axis = spawn_center_body
            .map(|b| b.rotation_axis)
            .unwrap_or_else(|| Vec3::new(0.0, 0.0, 1.0));
        let clockwise = spawn_center_body
            .map(|b| b.angular_speed > 0.0)
            .unwrap_or(true);

        // Spawn in the galaxy's local disk plane using the template plane basis.
        let a = fastrand::f32() * std::f32::consts::TAU;
        let (sin, cos) = a.sin_cos();
        let r = spawn_start_r + fastrand::f32() * (outer_r - spawn_start_r);

        // thickness: use template disk height factor if enabled, else small default
        let thickness = self.accretion_template.outer_radius
            * if self.accretion_template.disk_equilibrium_enabled {
                self.accretion_template.disk_equilibrium_scale_height_factor
            } else {
                0.01
            };
        let depth = (fastrand::f32() - 0.5) * thickness;
        let (u, v) = self.accretion_template.plane_basis(rotation_axis);
        let offset = (u * cos + v * sin) * r + rotation_axis * depth;
        let spawn_pos = spawn_center + offset;

        // Tangent direction for circular orbit in the local plane.
        let tangent = if clockwise {
            -rotation_axis.cross(offset).normalized()
        } else {
            rotation_axis.cross(offset).normalized()
        };

        let base_mass = mass_min + fastrand::f32() * (mass_max - mass_min);
        let element_mass_scale = atomic_number
            .and_then(element_by_atomic_number)
            .map(|element| {
                element.atomic_weight / 1.008 * self.config.element_orbital_mass_unit.max(0.0)
            })
            .unwrap_or(1.0);
        let mass = base_mass * element_mass_scale;
        let radius = mass.cbrt();

        let softening = self.config.epsilon * self.config.softening_scale_factor;
        let softening_sq = softening * softening;
        // Ensure the octree is built before querying acceleration. In unit tests spawn_accretion()
        // may be invoked directly without a prior attract() call, leaving the octree empty.
        if self.octree.nodes.is_empty() {
            let oct = Oct::new_containing(&self.bodies);
            let reserve_factor = self.config.performance_octree_reserve_factor.max(1.0);
            let reserve_nodes = ((self.bodies.len() as f32) * reserve_factor).ceil() as usize;
            let reserve_parents = ((self.bodies.len() as f32) * (reserve_factor * 0.25)).ceil() as usize;
            self.octree.reserve(reserve_nodes, reserve_parents);
            self.octree.clear(oct);
            for b in &self.bodies {
                let eff_mass = b.mass * Self::elemental_gravity_scale(b, &self.config);
                let momentum = Vec3::new(b.vel.x * eff_mass, b.vel.y * eff_mass, b.vel.z * eff_mass);
                self.octree.insert_delegate(b.pos, eff_mass, momentum, Vec3::zero());
            }
            self.octree.propagate();
        }
        let acc = self.octree.acc(spawn_pos, softening_sq);

        let projected_offset = offset - rotation_axis * offset.dot(rotation_axis);
        let orbital_radius = projected_offset.mag().max(f32::EPSILON);

        let orbital_speed = (acc.mag() * orbital_radius).sqrt();
        let vel = tangent * orbital_speed;

        let angular_speed = (self.config.spawn_angular_speed_base
            + fastrand::f32() * self.config.spawn_angular_speed_range
            + orbital_speed * 0.02)
            * self.config.spin_speed_multiplier;
        let mut body = Body::new(
            spawn_pos,
            vel,
            mass,
            radius,
            angular_speed,
            rotation_axis,
        );
        if let Some(atomic_number) = atomic_number {
            body = body.with_element(atomic_number, BodySegment::Accretion);
        }
        // inherit cluster membership from spawn center if available and insert via helper
        let cid = *self.body_cluster_ids.get(spawn_center_idx).unwrap_or(&0);
        self.add_body_with_cluster(body, Some(cid));
    }

    fn process_manual_spawns(&mut self) {
        let pending_spawns = std::mem::take(&mut *renderer::MANUAL_SPAWNS.lock());
        for request in pending_spawns {
            self.insert_manual_spawn(request);
        }
    }

    fn insert_manual_spawn(&mut self, request: renderer::ManualSpawnRequest) {
        let element_numbers = self.element_numbers_for_identity();
        let atomic_number = request.element_atomic_number.or_else(|| {
            if element_numbers.is_empty() {
                None
            } else {
                Some(element_numbers[fastrand::usize(0..element_numbers.len())])
            }
        });

        let mut body = Body::new(
            request.position,
            request.velocity,
            request.mass,
            request.mass.cbrt(),
            0.0,
            Vec3::new(0.0, 0.0, 1.0),
        );
        if let Some(atomic_number) = atomic_number {
            body = body.with_element(atomic_number, BodySegment::ManualSpawn);
        }
        // choose cluster id by nearest existing body if available, else default 0
        let cid = if !self.bodies.is_empty() && !self.body_cluster_ids.is_empty() {
            let mut best_idx = 0usize;
            let mut best_d2 = (self.bodies[0].pos - request.position).mag_sq();
            for (i, b) in self.bodies.iter().enumerate().skip(1) {
                let d2 = (b.pos - request.position).mag_sq();
                if d2 < best_d2 {
                    best_idx = i;
                    best_d2 = d2;
                }
            }
            *self.body_cluster_ids.get(best_idx).unwrap_or(&0)
        } else {
            0usize
        };
        self.add_body_with_cluster(body, Some(cid));
    }

    fn candidate_gas_collapse_detect(&self, body_idx: usize) -> Option<(usize, f32)> {
        // Lightweight, deterministic collapse candidate probe: returns (body_idx, suggested_star_mass)
        if body_idx >= self.bodies.len() { return None; }
        let b = &self.bodies[body_idx];
        use crate::body::BodySegment;
        if b.segment != BodySegment::Gas { return None; }
        if b.gas_density <= self.config.gas_ignition_density { return None; }
        // Coarse region cell coordinates for deterministic seeding
        let scale = self.config.gas_h_max.max(1.0);
        let xi = (b.pos.x / scale).floor() as i64;
        let yi = (b.pos.y / scale).floor() as i64;
        let zi = (b.pos.z / scale).floor() as i64;
        let seed = crate::astro::region_seed(xi, yi, zi, self.frame as u64, self.config.total_body_budget as u64 ^ 0xC0FFEEu64);
        let mut rng = fastrand::Rng::with_seed(seed);
        // Fraction of local gas to convert in a single collapse event (capped)
        let frag_frac = self.config.gas_condensation_strength.clamp(0.01, 0.9);
        let spawn_mass = (b.mass * frag_frac).min(self.config.element_orbital_mass_unit.max(1.0) * 100.0);
        // Sample a stellar mass from an IMF-like power-law (Salpeter ~2.35)
        let star_mass = crate::astro::sample_powerlaw_mass(&mut rng, self.config.element_orbital_mass_unit.max(0.01), spawn_mass.max(0.1), 2.35);
        Some((body_idx, star_mass))
    }

    /// Simple deterministic star lifecycle updater — lightweight, safe, and parameterized
    /// Uses StarState carried by Body.star_state (if present). When fuel is exhausted,
    /// core-collapse supernovae are scheduled for high-mass stars and low-mass stars become remnants.
    fn process_gas_collapse_and_spawn(&mut self) {
        use crate::body::BodySegment;
        use crate::astro::StarState;
        let mut spawns: Vec<(usize, f32)> = Vec::new();
        // Collect candidates without mutating bodies during iteration
        for idx in 0..self.bodies.len() {
            if let Some((bi, star_mass)) = self.candidate_gas_collapse_detect(idx) {
                spawns.push((bi, star_mass));
            }
        }

        for (gas_idx, star_mass) in spawns {
            if gas_idx >= self.bodies.len() { continue; }
            if self.bodies[gas_idx].segment != BodySegment::Gas { continue; }
            let available = self.bodies[gas_idx].mass.max(0.0);
            let convert_mass = star_mass.min(available * 0.8).max(0.0);
            if convert_mass < self.config.element_orbital_mass_unit * 0.05 {
                continue; // too small to bother
            }

            // drain gas mass and update local gas properties conservatively
            self.bodies[gas_idx].mass = (self.bodies[gas_idx].mass - convert_mass).max(0.0);
            self.bodies[gas_idx].gas_density = self.bodies[gas_idx].gas_density.max(0.0) * 0.5;

            // spawn a new star body near the gas parcel
            let spawn_pos = self.bodies[gas_idx].pos
                + Vec3::new((fastrand::f32() - 0.5) * 0.2, (fastrand::f32() - 0.5) * 0.2, (fastrand::f32() - 0.5) * 0.1)
                    * self.bodies[gas_idx].effective_radius().max(1.0);
            let spawn_vel = self.bodies[gas_idx].vel
                + Vec3::new((fastrand::f32() - 0.5) * 0.02, (fastrand::f32() - 0.5) * 0.02, 0.0);

            let mut star_body = Body::new(
                spawn_pos,
                spawn_vel,
                convert_mass,
                convert_mass.cbrt().max(0.5),
                0.0,
                Vec3::new(0.0, 0.0, 1.0),
            );
            star_body.segment = BodySegment::ManualSpawn;
            star_body.star_state = Some(StarState::new(convert_mass, self.bodies[gas_idx].gas_temperature.max(0.001)));
            // inherit cluster membership of the gas cell if available and insert via helper
            let cid = *self.body_cluster_ids.get(gas_idx).unwrap_or(&0);
            self.add_body_with_cluster(star_body, Some(cid));

            eprintln!("✨ Proto-star spawned from gas body {} mass={:.3}", self.bodies[gas_idx].body_id, convert_mass);
        }
    }

    fn update_star_evolution(&mut self) {
        use crate::astro::{StarStage, StarState};
        use crate::body::BodySegment;
        let mut sn_events: Vec<usize> = Vec::new();
        let dt = self.dt.max(0.0);
        for idx in 0..self.bodies.len() {
            if let Some(mut star) = self.bodies[idx].star_state {
                // Advance age and consume fuel proportional to luminosity (scaled small)
                star.age += dt;
                let burn = star.luminosity * dt * 1e-3; // small burn rate to keep timescales reasonable
                star.fuel -= burn;

                // Basic stage transitions (heuristic)
                if star.stage == StarStage::Protostar && star.age > 10.0 {
                    star.stage = StarStage::MainSequence;
                } else if star.stage == StarStage::MainSequence && star.age > (50.0 * (1.0 / star.mass.max(0.1))) {
                    star.stage = StarStage::RedGiant;
                } else if star.stage == StarStage::RedGiant && star.fuel <= 0.0 {
                    star.stage = StarStage::PostAGB;
                }

                // End-of-life decision
                if star.fuel <= 0.0 {
                    if star.mass >= 8.0 {
                        // schedule SN for this index
                        sn_events.push(idx);
                    } else {
                        // low-mass remnant (white-dwarf like): use neutron remnant placeholder
                        star.stage = StarStage::RemnantNeutron;
                    }
                    star.fuel = 0.0;
                }

                self.bodies[idx].star_state = Some(star);
            }
        }

        if !sn_events.is_empty() {
            self.process_supernovae_simple(&sn_events);
        }
    }

    /// Very lightweight supernova handling: reduces progenitor mass to a compact remnant,
    /// injects a modest fraction of ejecta mass and momentum into nearby gas bodies,
    /// and increases their gas_density / metallicity slightly. Intentionally conservative
    /// scaling to avoid destabilizing the N-body dynamics.
    fn process_supernovae_simple(&mut self, events: &[usize]) {
        use crate::body::BodySegment;
        for &idx in events {
            if idx >= self.bodies.len() { continue; }
            let progenitor = self.bodies[idx];
            let sn_mass = progenitor.mass * 0.8; // ejecta
            let core_mass = (progenitor.mass - sn_mass).max(0.05);

            // Set remnant properties
            let remnant_stage = if progenitor.star_state.map(|s| s.mass).unwrap_or(0.0) > 20.0 {
                crate::astro::StarStage::RemnantBlackHole
            } else {
                crate::astro::StarStage::RemnantNeutron
            };

            let remnant_state = crate::astro::StarState {
                mass: core_mass,
                age: 0.0,
                stage: remnant_stage,
                fuel: 0.0,
                core_mass: core_mass,
                metallicity: progenitor.star_state.map(|s| s.metallicity).unwrap_or(0.02),
                luminosity: 0.0,
            };

            self.bodies[idx].mass = core_mass;
            self.bodies[idx].star_state = Some(remnant_state);

            // Distribute a conservative fraction of ejecta to nearby gas bodies
            let ejecta_to_distribute = sn_mass * 0.6; // keep some mass untracked
            if ejecta_to_distribute <= 0.0 { continue; }
            let radius = 200.0_f32.max(progenitor.effective_radius() * 10.0);
            for j in 0..self.bodies.len() {
                if j == idx { continue; }
                let dist = (self.bodies[j].pos - progenitor.pos).mag();
                if dist <= radius {
                    if self.bodies[j].segment == BodySegment::Gas {
                        // spread fraction by distance (simple kernel)
                        let frac = (1.0 - (dist / radius)).clamp(0.0, 1.0) * 0.2; // modest fraction
                        let added_mass = ejecta_to_distribute * frac;
                        self.bodies[j].mass += added_mass;
                        self.bodies[j].gas_density += added_mass / (1.0 + self.bodies[j].effective_radius());
                        // metallicity bump stored in gas_temperature as a cheap channel? prefer to skip global invasive changes
                        self.bodies[j].gas_temperature += 0.01 * frac; // small energy injection
                        // small velocity kick radially outward
                        let mut _dir_vec = self.bodies[j].pos - progenitor.pos;
                        let dir = if _dir_vec.mag_sq() > 1e-12 { _dir_vec.normalized() } else { Vec3::zero() };
                        self.bodies[j].vel += dir * (10.0 * frac);
                    }
                }
            }

            eprintln!("💥 Supernova: body_id={} ejecta_mass={:.3} remnant_mass={:.3}", progenitor.body_id, sn_mass, core_mass);
        }
    }

    fn resolve(&mut self, i: usize, j: usize, center_merge_pair: &mut Option<(usize, usize)>) {
        if (i == self.center1_idx && j == self.center2_idx)
            || (i == self.center2_idx && j == self.center1_idx)
        {
            if self.form_molecular_bond(i, j) {
                return;
            }
            *center_merge_pair = Some((i, j));
            return;
        }

        let b1 = self.bodies[i];
        let b2 = self.bodies[j];

        let p1 = b1.pos;
        let p2 = b2.pos;

        let r1 = b1.effective_radius();
        let r2 = b2.effective_radius();

        let d = p2 - p1;
        let r = r1 + r2;

        if d.mag_sq() > r * r {
            return;
        }
        if self.form_molecular_bond(i, j) {
            return;
        }

        let v1 = b1.vel;
        let v2 = b2.vel;

        let v = v2 - v1;

        let d_dot_v = d.dot(v);

        let m1 = b1.mass;
        let m2 = b2.mass;

        let weight1 = m2 / (m1 + m2);
        let weight2 = m1 / (m1 + m2);

        if d_dot_v >= 0.0 && d != Vec3::zero() {
            let tmp = d * (r / d.mag() - 1.0);
            self.bodies[i].pos -= weight1 * tmp;
            self.bodies[j].pos += weight2 * tmp;
            return;
        }

        let v_sq = v.mag_sq();
        let d_sq = d.mag_sq();
        let r_sq = r * r;

        let t = (d_dot_v + (d_dot_v * d_dot_v - v_sq * (d_sq - r_sq)).max(0.0).sqrt()) / v_sq;

        self.bodies[i].pos -= v1 * t;
        self.bodies[j].pos -= v2 * t;

        let p1 = self.bodies[i].pos;
        let p2 = self.bodies[j].pos;
        let d = p2 - p1;
        let d_dot_v = d.dot(v);
        let d_sq = d.mag_sq();

        let binding_response = if self.config.enable_elemental_binding_response {
            match (b1.element_atomic_number, b2.element_atomic_number) {
                (Some(first_number), Some(second_number)) => {
                    let tendency = binding_tendency(
                        element_by_atomic_number(first_number).unwrap(),
                        element_by_atomic_number(second_number).unwrap(),
                    );
                    let response_strength = match tendency {
                        BindingTendency::Inert => 0.0,
                        BindingTendency::Metallic => 0.25,
                        BindingTendency::NonPolarCovalent => 0.35,
                        BindingTendency::PolarCovalent => 0.7,
                        BindingTendency::Ionic => 1.0,
                    };
                    (1.0 - self
                        .config
                        .elemental_binding_response_strength
                        .clamp(0.0, 1.0)
                        * response_strength)
                        .clamp(0.0, 1.0)
                }
                _ => 1.0,
            }
        } else {
            1.0
        };
        let tmp = d * (1.5 * binding_response * d_dot_v / d_sq);
        let v1 = v1 + tmp * weight1;
        let v2 = v2 - tmp * weight2;

        self.bodies[i].vel = v1;
        self.bodies[j].vel = v2;
        self.bodies[i].pos += v1 * t;
        self.bodies[j].pos += v2 * t;
    }

    fn merge_center_particles(&mut self, i: usize, j: usize) {
        let mut first = i;
        let mut second = j;
        if first > second {
            std::mem::swap(&mut first, &mut second);
        }

        let b1 = self.bodies[first];
        let b2 = self.bodies[second];
        let total_mass = b1.mass + b2.mass;
        let merged_pos = (b1.pos * b1.mass + b2.pos * b2.mass) / total_mass;
        let merged_vel = (b1.vel * b1.mass + b2.vel * b2.mass) / total_mass;

        let orbital_axis = (b2.pos - b1.pos).cross(b2.vel - b1.vel);
        let axis = if orbital_axis.mag_sq() > 1e-8 {
            orbital_axis.normalized()
        } else {
            let axis_sum = b1.rotation_axis * b1.mass + b2.rotation_axis * b2.mass;
            if axis_sum.mag_sq() > 1e-8 {
                axis_sum.normalized()
            } else {
                Vec3::new(0.0, 0.0, 1.0)
            }
        };

        let merged_radius = (b1.base_radius.powi(3) + b2.base_radius.powi(3)).cbrt();
        let merged_angular_speed =
            (b1.angular_speed.abs() * b1.mass + b2.angular_speed.abs() * b2.mass) / total_mass;
        let mut merged_body = Body::new(
            merged_pos,
            merged_vel,
            total_mass,
            merged_radius,
            merged_angular_speed,
            axis,
        );
        let retained_element = match (b1.element_atomic_number, b2.element_atomic_number) {
            (Some(first_number), Some(second_number)) => {
                let first_score = element_by_atomic_number(first_number)
                    .map(|element| element.core_suitability() * b1.mass)
                    .unwrap_or(0.0);
                let second_score = element_by_atomic_number(second_number)
                    .map(|element| element.core_suitability() * b2.mass)
                    .unwrap_or(0.0);
                Some(if first_score >= second_score {
                    first_number
                } else {
                    second_number
                })
            }
            (Some(atomic_number), None) | (None, Some(atomic_number)) => Some(atomic_number),
            (None, None) => None,
        };
        if let Some(atomic_number) = retained_element {
            merged_body = merged_body.with_element(atomic_number, BodySegment::Core);
        }
        merged_body.body_id = b1.body_id;
        merged_body.molecule_id = b1.molecule_id.or(b2.molecule_id);
        self.bodies[first] = merged_body;
        if let Some(removed_body_id) = self.remove_body_at(second) {
            self.molecular_bonds.retain(|bond| {
                bond.first_body_id != removed_body_id && bond.second_body_id != removed_body_id
            });
        }

        self.center1_idx = first;
        self.center2_idx = usize::MAX;
        self.equatorial_plane_normal = axis;
        self.equatorial_plane_center = merged_pos;

        eprintln!(
            "🔗 Center merge: mass={} pos={:?} axis={:?} inflow-plane-normal={:?}",
            total_mass, merged_pos, axis, self.equatorial_plane_normal
        );
    }
}

fn molecule_size(bodies: &[Body], molecule_id: Option<u64>) -> usize {
    match molecule_id {
        Some(molecule_id) => bodies
            .iter()
            .filter(|body| body.molecule_id == Some(molecule_id))
            .count(),
        None => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::Simulation;
    use crate::{body::BodySegment, config::InformationsConfig, renderer::ManualSpawnRequest};
    use ultraviolet::Vec3;

    #[test]
    fn simulation_new_uses_galaxy_count() {
        let mut cfg = InformationsConfig::default();
        cfg.n = 10; // small per-galaxy particle count for fast tests
        cfg.galaxy_count = 6; // even number -> 3 pairs / 6 galaxies total
        cfg.enable_galactic_atom_simulation = false;
        let sim = Simulation::new(&cfg);
        assert_eq!(sim.galaxy_start_indices.len(), cfg.galaxy_count);
    }

    #[test]
    fn spawn_accretion_alignment_matches_galaxy_plane() {
        let mut cfg = InformationsConfig::default();
        cfg.n = 10;
        cfg.galaxy_count = 2;
        cfg.accretion_spawn_rate = 1.0;
        let mut sim = Simulation::new(&cfg);
        // Ensure the runtime template will always spawn for the test
        sim.accretion_template.accretion_spawn_rate = 1.0;
        let spawn_center_idx = sim.center1_idx;
        let rotation_axis_before = sim.bodies[spawn_center_idx].rotation_axis;
        let old_len = sim.bodies.len();
        sim.spawn_accretion();
        assert_eq!(sim.bodies.len(), old_len + 1);
        let new_body = sim.bodies.last().unwrap();
        // Allow a tiny floating-point tolerance for rotation axis equality
        let diff = (new_body.rotation_axis - rotation_axis_before).mag();
        assert!(diff < 1e-6, "rotation axis differs by {} (>1e-6)", diff);
    }

    #[test]
    fn cache_galaxies_slices_into_galaxy_states() {
        let mut cfg = InformationsConfig::default();
        cfg.n = 6;
        cfg.galaxy_count = 6; // 3 pairs -> 6 galaxies
        cfg.enable_galactic_atom_simulation = false;
        let sim = Simulation::new(&cfg);
        // galaxy_start_indices should match configured galaxy_count and cache_galaxies should have created that many snapshots
        assert_eq!(sim.galaxy_start_indices.len(), cfg.galaxy_count);
        assert_eq!(sim.galaxies.len(), cfg.galaxy_count);
        let counted: usize = sim.galaxies.iter().map(|g| g.bodies.len()).sum();
        assert_eq!(counted, sim.bodies.len());
    }

    #[test]
    fn atom_mode_sets_single_galaxy_start_index() {
        let mut cfg = InformationsConfig::default();
        cfg.enable_galactic_atom_simulation = true;
        cfg.element_atomic_count = 12;
        let sim = Simulation::new(&cfg);
        assert_eq!(sim.galaxy_start_indices, vec![0usize]);
        assert_eq!(sim.galaxies.len(), 1);
    }

    #[test]
    fn galactic_atom_mode_respects_selected_elements_and_body_budget() {
        let mut config = InformationsConfig::default();
        config.enable_galactic_atom_simulation = true;
        config.element_atomic_numbers = vec![6, 8];
        config.element_atomic_count = 12;

        let simulation = Simulation::new(&config);

        assert_eq!(simulation.bodies.len(), 12);
        assert_eq!(simulation.center2_idx, usize::MAX);
        assert_eq!(
            simulation
                .bodies
                .iter()
                .filter(|body| body.segment == BodySegment::Core)
                .count(),
            2
        );
        assert!(simulation.bodies.iter().all(|body| {
            matches!(body.element_atomic_number, Some(6 | 8))
                && body.segment != BodySegment::Unspecified
        }));
    }

    #[test]
    fn paired_galaxy_default_remains_unlabelled() {
        let mut config = InformationsConfig::default();
        config.n = 8;
        config.galaxy_count = 2;
        config.enable_elemental_galaxy_pair_mode = false;

        let simulation = Simulation::new(&config);

        assert_eq!(simulation.bodies.len(), 18);
        assert!(simulation
            .bodies
            .iter()
            .all(|body| body.element_atomic_number.is_none()));
    }

    #[test]
    fn unlike_element_collision_forms_a_persistent_bond() {
        let mut config = InformationsConfig::default();
        config.n = 8;
        config.enable_molecular_bonding = true;
        config.molecule_max_bodies = 4;
        let mut simulation = Simulation::new(&config);
        simulation.bodies[1].element_atomic_number = Some(1);
        simulation.bodies[1].segment = BodySegment::Orbital;
        simulation.bodies[1].pos = Vec3::zero();
        simulation.bodies[1].vel = Vec3::zero();
        simulation.bodies[2].element_atomic_number = Some(8);
        simulation.bodies[2].segment = BodySegment::Orbital;
        simulation.bodies[2].pos = Vec3::new(0.2, 0.0, 0.0);
        simulation.bodies[2].vel = Vec3::new(-1.0, 0.0, 0.0);

        simulation.resolve(1, 2, &mut None);

        assert_eq!(simulation.molecular_bonds.len(), 1);
        assert!(simulation.bodies[1].molecule_id.is_some());
        assert_eq!(
            simulation.bodies[1].molecule_id,
            simulation.bodies[2].molecule_id
        );
    }

    #[test]
    fn molecular_spring_force_conserves_pair_momentum_and_breaks_at_threshold() {
        let mut config = InformationsConfig::default();
        config.n = 8;
        config.enable_molecular_bonding = true;
        config.molecule_max_bodies = 4;
        let mut simulation = Simulation::new(&config);
        simulation.bodies[1].element_atomic_number = Some(1);
        simulation.bodies[1].segment = BodySegment::Orbital;
        simulation.bodies[1].pos = Vec3::zero();
        simulation.bodies[1].vel = Vec3::zero();
        simulation.bodies[2].element_atomic_number = Some(8);
        simulation.bodies[2].segment = BodySegment::Orbital;
        simulation.bodies[2].pos = Vec3::new(0.2, 0.0, 0.0);
        simulation.bodies[2].vel = Vec3::new(-1.0, 0.0, 0.0);
        simulation.resolve(1, 2, &mut None);

        simulation.bodies[1].vel = Vec3::zero();
        simulation.bodies[2].vel = Vec3::zero();
        simulation.bodies[1].acc = Vec3::zero();
        simulation.bodies[2].acc = Vec3::zero();
        simulation.apply_molecular_bond_forces();

        let total_force = simulation.bodies[1].acc * simulation.bodies[1].mass
            + simulation.bodies[2].acc * simulation.bodies[2].mass;
        assert!(total_force.mag() < 1.0e-5);
        assert_eq!(simulation.molecular_bonds.len(), 1);

        simulation.bodies[2].vel = Vec3::new(100.0, 0.0, 0.0);
        simulation.apply_molecular_bond_forces();
        assert!(simulation.molecular_bonds.is_empty());
        assert!(simulation.bodies[1].molecule_id.is_none());
        assert!(simulation.bodies[2].molecule_id.is_none());
    }

    #[test]
    fn molecule_composition_resolves_known_compounds() {
        let mut config = InformationsConfig::default();
        config.n = 8;
        config.molecule_identify_compounds = true;
        let mut simulation = Simulation::new(&config);
        for (index, atomic_number) in [(1, 1), (2, 1), (3, 8)] {
            simulation.bodies[index].element_atomic_number = Some(atomic_number);
            simulation.bodies[index].molecule_id = Some(42);
        }

        assert_eq!(simulation.identified_compound(42).unwrap().formula, "H2O");
    }

    #[test]
    fn manual_element_spawn_preserves_selection_and_gets_a_stable_id() {
        let mut config = InformationsConfig::default();
        config.n = 8;
        let mut simulation = Simulation::new(&config);
        let previous_count = simulation.bodies.len();

        simulation.insert_manual_spawn(ManualSpawnRequest {
            position: Vec3::new(12.0, -4.0, 0.0),
            velocity: Vec3::new(0.0, 2.0, 0.0),
            mass: 3.0,
            element_atomic_number: Some(8),
        });

        let spawned = simulation.bodies.last().unwrap();
        assert_eq!(simulation.bodies.len(), previous_count + 1);
        assert_eq!(spawned.element_atomic_number, Some(8));
        assert_eq!(spawned.segment, BodySegment::ManualSpawn);
        assert_ne!(spawned.body_id, 0);
    }

    #[test]
    fn roles_update_normalizes_and_applies_small_acceleration() {
        let mut config = InformationsConfig::default();
        config.n = 8;
        config.enable_flow_dynamics_model = true;
        config.role_update_interval = 1;
        let mut simulation = Simulation::new(&config);

        // Trigger role signals on a selected body
        simulation.bodies[1].gas_density = 10.0;
        simulation.bodies[1].accretion_ejection_bias = 0.2;
        simulation.bodies[1].magnetization = 0.5;
        simulation.bodies[1].acc = Vec3::zero();

        simulation.update_roles();

        let rw = simulation.bodies[1].roles;
        let sum = rw.accretor
            + rw.transporter
            + rw.heater
            + rw.densifier
            + rw.wind
            + rw.jet
            + rw.escapee
            + rw.reservoir;
        assert!((sum - 1.0).abs() < 1e-6, "role weights must sum to 1");
        assert!(
            rw.accretor > 0.0
                || rw.densifier > 0.0
                || rw.transporter > 0.0
                || rw.jet > 0.0
                || rw.wind > 0.0,
            "expected at least one non-reservoir role when gas/magnetization present"
        );
        assert!(
            simulation.bodies[1].acc.mag() > 0.0,
            "role effects should have modified acceleration"
        );
    }

    #[test]
    fn roles_default_to_reservoir_when_no_triggers() {
        let mut config = InformationsConfig::default();
        config.n = 8;
        config.enable_flow_dynamics_model = true;
        let mut simulation = Simulation::new(&config);

        simulation.bodies[2].gas_density = 0.0;
        simulation.bodies[2].accretion_ejection_bias = 0.0;
        simulation.bodies[2].magnetization = 0.0;
        simulation.bodies[2].acc = Vec3::zero();

        simulation.update_roles();

        let rw = simulation.bodies[2].roles;
        assert!(rw.reservoir.is_finite());
        assert!(
            rw.reservoir > 0.9,
            "reservoir should dominate when no triggers"
        );
    }
}
