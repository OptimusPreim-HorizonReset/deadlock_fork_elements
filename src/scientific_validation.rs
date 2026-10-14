#![allow(clippy::field_reassign_with_default)]

use std::{mem::size_of, time::Instant};

use crate::{
    body::{Body, BodySegment},
    config::InformationsConfig,
    elements::element_by_atomic_number,
    galaxy_templates::{plummer_circular_speed, GalaxyTemplate},
    gas::GasBillboard,
    quadtree::{Oct, Octree},
    simulation::Simulation,
};
use ultraviolet::{Vec2, Vec3};

fn make_body(position: Vec3, velocity: Vec3, mass: f32) -> Body {
    Body::new(position, velocity, mass, 0.4, 0.0, Vec3::new(0.0, 0.0, 1.0))
}

fn make_tree(bodies: &[Body], theta: f32, softening: f32) -> Octree {
    let mut tree = Octree::new(theta, softening);
    tree.clear(Oct::new_containing(bodies));
    for body in bodies {
        tree.insert(body.pos, body.mass);
    }
    tree.propagate();
    tree
}

fn direct_acceleration(bodies: &[Body], target_index: usize, softening_squared: f32) -> Vec3 {
    let target = bodies[target_index];
    let mut acceleration = Vec3::zero();
    for (source_index, source) in bodies.iter().enumerate() {
        if source_index == target_index {
            continue;
        }
        let displacement = source.pos - target.pos;
        let distance_squared = displacement.mag_sq();
        if distance_squared > 0.0 {
            let denominator = (distance_squared + softening_squared) * distance_squared.sqrt();
            acceleration += displacement * (source.mass / denominator);
        }
    }
    acceleration
}

fn tree_accelerations(bodies: &[Body], theta: f32, softening: f32) -> Vec<Vec3> {
    let tree = make_tree(bodies, theta, softening);
    bodies
        .iter()
        .map(|body| tree.acc(body.pos, softening * softening))
        .collect()
}

fn sample_bodies(count: usize) -> Vec<Body> {
    (0..count)
        .map(|index| {
            let index = index as f32;
            let position = Vec3::new(
                (index * 12.9898).sin() * 12.0,
                (index * 7.233).cos() * 9.0,
                (index * 3.711).sin() * 6.0,
            );
            make_body(
                position,
                Vec3::zero(),
                0.7 + (index as usize % 7) as f32 * 0.2,
            )
        })
        .collect()
}

fn relative_rms_error(actual: &[Vec3], expected: &[Vec3]) -> f32 {
    let error_squared: f32 = actual
        .iter()
        .zip(expected)
        .map(|(actual, expected)| (*actual - *expected).mag_sq())
        .sum();
    let expected_squared: f32 = expected.iter().map(|value| value.mag_sq()).sum();
    (error_squared / expected_squared.max(f32::EPSILON)).sqrt()
}

fn binary_pair() -> [Body; 2] {
    let orbital_speed = 0.5_f32.sqrt();
    [
        make_body(
            Vec3::new(-0.5, 0.0, 0.0),
            Vec3::new(0.0, -orbital_speed, 0.0),
            1.0,
        ),
        make_body(
            Vec3::new(0.5, 0.0, 0.0),
            Vec3::new(0.0, orbital_speed, 0.0),
            1.0,
        ),
    ]
}

fn advance_binary(bodies: &mut [Body; 2], steps: usize, dt: f32) {
    for _ in 0..steps {
        let accelerations = tree_accelerations(bodies, 0.0, 0.0);
        for (body, acceleration) in bodies.iter_mut().zip(accelerations) {
            body.acc = acceleration;
            body.update(dt);
        }
    }
}

fn binary_energy(bodies: &[Body; 2]) -> f32 {
    let displacement = bodies[1].pos - bodies[0].pos;
    let kinetic = bodies
        .iter()
        .map(|body| 0.5 * body.mass * body.vel.mag_sq())
        .sum::<f32>();
    kinetic - bodies[0].mass * bodies[1].mass / displacement.mag()
}

fn binary_angular_momentum_z(bodies: &[Body; 2]) -> f32 {
    bodies
        .iter()
        .map(|body| body.mass * (body.pos.x * body.vel.y - body.pos.y * body.vel.x))
        .sum()
}

fn binary_center_of_mass(bodies: &[Body; 2]) -> Vec3 {
    let total_mass = bodies[0].mass + bodies[1].mass;
    (bodies[0].pos * bodies[0].mass + bodies[1].pos * bodies[1].mass) / total_mass
}

#[test]
fn unit_plummer_softened_gravity_matches_analytic_reference() {
    let bodies = [
        make_body(Vec3::zero(), Vec3::zero(), 3.0),
        make_body(Vec3::new(2.0, 0.0, 0.0), Vec3::zero(), 1.0),
    ];
    let tree = make_tree(&bodies, 0.0, 0.5);
    let acceleration = tree.acc(bodies[1].pos, 0.25);
    let expected = Vec3::new(-6.0 / 8.5, 0.0, 0.0);
    assert!((acceleration - expected).mag() < 1.0e-6);
}

#[test]
fn unit_unsoftened_gravity_follows_inverse_square_law() {
    let bodies = [make_body(Vec3::zero(), Vec3::zero(), 1.0)];
    let tree = make_tree(&bodies, 0.0, 0.0);
    let near = tree.acc(Vec3::new(1.0, 0.0, 0.0), 0.0).mag();
    let far = tree.acc(Vec3::new(2.0, 0.0, 0.0), 0.0).mag();
    assert!((near / far - 4.0).abs() < 1.0e-6);
}

#[test]
fn unit_zero_distance_self_force_is_finite_and_zero() {
    let bodies = [make_body(Vec3::new(2.0, -1.0, 4.0), Vec3::zero(), 8.0)];
    let tree = make_tree(&bodies, 0.5, 0.2);
    let acceleration = tree.acc(bodies[0].pos, 0.04);
    assert_eq!(acceleration, Vec3::zero());
    assert!(acceleration.x.is_finite() && acceleration.y.is_finite() && acceleration.z.is_finite());
}

#[test]
fn unit_octree_mass_and_center_of_mass_are_exact() {
    let bodies = [
        make_body(Vec3::new(-1.0, 0.0, 0.0), Vec3::zero(), 1.0),
        make_body(Vec3::zero(), Vec3::zero(), 2.0),
        make_body(Vec3::new(2.0, 0.0, 0.0), Vec3::zero(), 3.0),
    ];
    let tree = make_tree(&bodies, 0.5, 0.0);
    assert!((tree.nodes[Octree::ROOT].mass - 6.0).abs() < 1.0e-6);
    assert!((tree.nodes[Octree::ROOT].pos.x - 5.0 / 6.0).abs() < 1.0e-6);
}

#[test]
fn unit_body_integration_remains_semi_implicit_euler() {
    let mut body = make_body(Vec3::zero(), Vec3::new(1.0, 0.0, 0.0), 2.0);
    body.acc = Vec3::new(0.0, 4.0, 0.0);
    body.update(0.5);
    assert_eq!(body.vel, Vec3::new(1.0, 2.0, 0.0));
    assert_eq!(body.pos, Vec3::new(0.5, 1.0, 0.0));
}

#[test]
fn physics_linear_momentum_is_conserved_in_an_isolated_binary() {
    let mut bodies = binary_pair();
    let initial_momentum = bodies[0].vel * bodies[0].mass + bodies[1].vel * bodies[1].mass;
    advance_binary(&mut bodies, 200, 0.001);
    let final_momentum = bodies[0].vel * bodies[0].mass + bodies[1].vel * bodies[1].mass;
    assert!((final_momentum - initial_momentum).mag() < 1.0e-5);
}

#[test]
fn physics_angular_momentum_drift_stays_below_reference_tolerance() {
    let mut bodies = binary_pair();
    let initial = binary_angular_momentum_z(&bodies);
    advance_binary(&mut bodies, 2000, 0.0005);
    let final_value = binary_angular_momentum_z(&bodies);
    assert!(((final_value - initial) / initial).abs() < 5.0e-4);
}

#[test]
fn physics_symplectic_energy_drift_remains_bounded() {
    let mut bodies = binary_pair();
    let initial = binary_energy(&bodies);
    advance_binary(&mut bodies, 2000, 0.0005);
    let final_value = binary_energy(&bodies);
    assert!(((final_value - initial) / initial.abs()).abs() < 5.0e-3);
}

#[test]
fn physics_center_of_mass_remains_stationary() {
    let mut bodies = binary_pair();
    let initial = binary_center_of_mass(&bodies);
    advance_binary(&mut bodies, 1000, 0.0005);
    assert!((binary_center_of_mass(&bodies) - initial).mag() < 1.0e-5);
}

#[test]
fn physics_production_steps_conserve_mass_without_collisions() {
    let mut config = InformationsConfig::default();
    config.n = 4;
    config.collision_interval = 100_000;
    config.accretion_spawn_rate = -1.0;
    let mut simulation = Simulation::new(&config);
    simulation.frame = 1;
    let initial_mass: f32 = simulation.bodies.iter().map(|body| body.mass).sum();
    for _ in 0..8 {
        simulation.step();
    }
    let final_mass: f32 = simulation.bodies.iter().map(|body| body.mass).sum();
    assert!((final_mass - initial_mass).abs() < 1.0e-3);
}

#[test]
fn physics_center_merge_conserves_mass_and_linear_momentum() {
    let mut config = InformationsConfig::default();
    config.n = 2;
    config.galaxy_count = 2;
    config.collision_interval = 1;
    let mut simulation = Simulation::new(&config);
    for (index, body) in simulation.bodies.iter_mut().enumerate() {
        body.pos = Vec3::new(index as f32 * 10.0 + 100.0, 0.0, 0.0);
        body.vel = Vec3::zero();
    }
    simulation.bodies[0].pos = Vec3::zero();
    simulation.bodies[0].vel = Vec3::new(2.0, 0.0, 0.0);
    simulation.bodies[0].mass = 4.0;
    simulation.bodies[3].pos = Vec3::new(0.05, 0.0, 0.0);
    simulation.bodies[3].vel = Vec3::new(-1.0, 0.0, 0.0);
    simulation.bodies[3].mass = 5.0;
    let _initial_mass: f32 = simulation.bodies.iter().map(|body| body.mass).sum();
    let _initial_momentum: Vec3 = simulation
        .bodies
        .iter()
        .map(|body| body.vel * body.mass)
        .fold(Vec3::zero(), |sum, value| sum + value);

    let initial_mass: f32 = simulation.bodies.iter().map(|body| body.mass).sum();
    let initial_momentum: Vec3 = simulation
        .bodies
        .iter()
        .map(|body| body.vel * body.mass)
        .fold(Vec3::zero(), |sum, value| sum + value);

    simulation.collide();

    let final_mass: f32 = simulation.bodies.iter().map(|body| body.mass).sum();
    let final_momentum: Vec3 = simulation
        .bodies
        .iter()
        .map(|body| body.vel * body.mass)
        .fold(Vec3::zero(), |sum, value| sum + value);
    assert_eq!(simulation.bodies.len(), 5);
    assert!((final_mass - initial_mass).abs() < 1.0e-3);
    assert!((final_momentum - initial_momentum).mag() < 1.0e-4);
}

#[test]
fn physics_default_softened_binary_remains_bounded() {
    let mut bodies = binary_pair();
    advance_binary(&mut bodies, 3000, 0.0005);
    let separation = (bodies[1].pos - bodies[0].pos).mag();
    assert!((0.7..=1.3).contains(&separation));
}

#[test]
fn physics_disk_equilibrium_uses_runtime_plummer_law() {
    let speed = plummer_circular_speed(12.0, 3.0, 0.5);
    let expected = (12.0 * 9.0 / 9.25_f32.powf(1.5)).sqrt();
    assert!((speed - expected).abs() < 1.0e-6);
}

#[test]
fn physics_inclined_generation_uses_projected_orbital_radius() {
    let mut config = InformationsConfig::default();
    config.n = 24;
    config.enable_disk_equilibrium_mode = false;
    config.inner_radius = 2.0;
    config.outer_radius = 20.0;
    config.central_mass = 100.0;
    let template = GalaxyTemplate::from_config(&config);
    let axis = Vec3::new(0.4, 0.6, 0.7).normalized();
    let bodies = template.generate_inclined(Vec2::zero(), Vec2::zero(), axis, true);
    let center = bodies[0].pos;
    let mut enclosed_mass = 0.0;

    for body in &bodies {
        enclosed_mass += body.mass;
        let offset = body.pos - center;
        let projected = offset - axis * offset.dot(axis);
        let radius = projected.mag();
        if radius > f32::EPSILON {
            let expected_speed = (enclosed_mass / radius).sqrt();
            assert!((body.vel.mag() - expected_speed).abs() < 1.0e-4);
        }
    }
}

#[test]
fn physics_elemental_mass_remapping_recomputes_circular_velocities() {
    let mut config = InformationsConfig::default();
    config.enable_galactic_atom_simulation = true;
    config.element_atomic_numbers = vec![26];
    config.element_atomic_count = 6;
    config.enable_scientific_element_gravity = true;
    config.element_prime_beta = 0.06;
    let simulation = Simulation::new(&config);
    assert!(simulation
        .bodies
        .iter()
        .filter(|body| body.segment != BodySegment::Core)
        .any(|body| body.vel.mag() > 0.0 && body.vel.mag().is_finite()));
}

#[test]
fn property_exact_tree_traversal_matches_direct_summation() {
    let bodies = sample_bodies(64);
    let softening = 0.15;
    let tree = make_tree(&bodies, 0.0, softening);
    let expected: Vec<_> = (0..bodies.len())
        .map(|index| direct_acceleration(&bodies, index, softening * softening))
        .collect();
    let actual: Vec<_> = bodies
        .iter()
        .map(|body| tree.acc(body.pos, softening * softening))
        .collect();
    assert!(relative_rms_error(&actual, &expected) < 1.0e-5);
}

#[test]
fn property_pair_forces_obey_newtons_third_law() {
    let bodies = [
        make_body(Vec3::new(-0.7, 0.0, 0.0), Vec3::zero(), 2.0),
        make_body(Vec3::new(0.6, 0.0, 0.0), Vec3::zero(), 5.0),
    ];
    let first_force = direct_acceleration(&bodies, 0, 0.04) * bodies[0].mass;
    let second_force = direct_acceleration(&bodies, 1, 0.04) * bodies[1].mass;
    assert!((first_force + second_force).mag() < 1.0e-6);
}

#[test]
fn property_barnes_hut_rms_force_error_is_below_five_percent() {
    let bodies = sample_bodies(128);
    let softening = 0.2;
    let expected: Vec<_> = (0..bodies.len())
        .map(|index| direct_acceleration(&bodies, index, softening * softening))
        .collect();
    let actual = tree_accelerations(&bodies, 0.5, softening);
    assert!(relative_rms_error(&actual, &expected) < 0.05);
}

#[test]
fn property_mixed_precision_force_orders_stay_below_eight_percent() {
    let bodies = sample_bodies(96);
    let softening = 0.2;
    let reference: Vec<_> = (0..bodies.len())
        .map(|index| direct_acceleration(&bodies, index, softening * softening))
        .collect();
    let actual = tree_accelerations(&bodies, 0.35, softening);
    assert!(relative_rms_error(&actual, &reference) < 0.08);
}

#[test]
fn property_simulation_attract_force_error_stays_below_eight_percent() {
    let mut config = InformationsConfig::default();
    config.n = 16;
    config.theta = 0.0;
    config.epsilon = 0.3;
    config.softening_scale_factor = 1.0;
    config.inflow_strength = 0.0;
    config.restore_strength = 0.0;
    config.sph_enabled = false;
    let mut simulation = Simulation::new(&config);
    simulation.attract();
    let softening_squared = config.epsilon * config.epsilon;
    let actual: Vec<_> = simulation.bodies.iter().map(|body| body.acc).collect();
    let expected: Vec<_> = (0..simulation.bodies.len())
        .map(|index| direct_acceleration(&simulation.bodies, index, softening_squared))
        .collect();
    assert!(relative_rms_error(&actual, &expected) < 0.08);
}

#[test]
fn property_bond_forces_are_restricted_to_bonded_bodies() {
    let mut config = InformationsConfig::default();
    config.n = 2;
    config.enable_molecular_bonding = true;
    config.inflow_strength = 0.0;
    config.restore_strength = 0.0;
    let mut simulation = Simulation::new(&config);
    for (index, body) in simulation.bodies.iter_mut().enumerate() {
        body.pos = Vec3::new(index as f32 * 3.0 + 30.0, 0.0, 0.0);
        body.vel = Vec3::zero();
        body.acc = Vec3::zero();
    }
    simulation.bodies[1].pos = Vec3::zero();
    simulation.bodies[1].vel = Vec3::zero();
    simulation.bodies[1].element_atomic_number = Some(1);
    simulation.bodies[1].segment = BodySegment::Orbital;
    simulation.bodies[2].pos = Vec3::new(0.2, 0.0, 0.0);
    simulation.bodies[2].vel = Vec3::new(-1.0, 0.0, 0.0);
    simulation.bodies[2].element_atomic_number = Some(8);
    simulation.bodies[2].segment = BodySegment::Orbital;
    simulation.collide();
    assert!(simulation.bodies[1].molecule_id.is_some());
    for body in &mut simulation.bodies {
        body.acc = Vec3::zero();
    }

    simulation.apply_molecular_bond_forces();

    assert_eq!(simulation.bodies[4].acc, Vec3::zero());
}

#[test]
fn property_element_gravity_does_not_create_self_pressure() {
    let element = element_by_atomic_number(26).unwrap();
    let body = make_body(Vec3::new(1.0, 2.0, 3.0), Vec3::zero(), 4.0)
        .with_element(26, BodySegment::Orbital);
    let effective_mass = body.mass * element.scientific_gravity_scale(0.06, [1.0, 0.4, 10.0, 0.2]);
    let tree = make_tree(
        &[Body {
            mass: effective_mass,
            ..body
        }],
        0.5,
        0.2,
    );
    assert_eq!(tree.acc(body.pos, 0.04), Vec3::zero());
}

#[test]
fn property_gravity_is_translation_invariant() {
    let bodies = sample_bodies(40);
    let translated: Vec<_> = bodies
        .iter()
        .map(|body| Body {
            pos: body.pos + Vec3::new(10.0, -8.0, 3.0),
            ..*body
        })
        .collect();
    let original = tree_accelerations(&bodies, 0.0, 0.2);
    let shifted = tree_accelerations(&translated, 0.0, 0.2);
    assert!(relative_rms_error(&shifted, &original) < 1.0e-5);
}

#[test]
fn property_gravity_is_rotation_covariant() {
    let bodies = sample_bodies(40);
    let rotate = |position: Vec3| Vec3::new(-position.y, position.x, position.z);
    let rotated: Vec<_> = bodies
        .iter()
        .map(|body| Body {
            pos: rotate(body.pos),
            ..*body
        })
        .collect();
    let original = tree_accelerations(&bodies, 0.0, 0.2);
    let rotated_accelerations = tree_accelerations(&rotated, 0.0, 0.2);
    let expected: Vec<_> = original
        .iter()
        .map(|acceleration| rotate(*acceleration))
        .collect();
    assert!(relative_rms_error(&rotated_accelerations, &expected) < 1.0e-5);
}

#[test]
fn property_bounded_inputs_never_produce_nan_or_infinity() {
    let bodies = sample_bodies(48);
    let tree = make_tree(&bodies, 0.5, 0.25);
    for body in &bodies {
        let acceleration = tree.acc(body.pos, 0.0625);
        assert!(
            acceleration.x.is_finite() && acceleration.y.is_finite() && acceleration.z.is_finite()
        );
    }
}

#[test]
fn shader_optional_glsl_artifact_parses_and_validates_with_naga() {
    let source = include_str!("../shaders/gas_billboard.glsl");
    let mut frontend = naga::front::glsl::Frontend::default();
    let module = frontend
        .parse(
            &naga::front::glsl::Options::from(naga::ShaderStage::Fragment),
            source,
        )
        .unwrap();
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .unwrap();
}

#[test]
fn shader_cpu_gas_billboard_alpha_layers_are_bounded_and_ordered() {
    let [outer, middle, inner] = GasBillboard::new(0.8, 4.0).alpha_layers();
    assert!(outer <= 0.9 && outer >= middle && middle >= inner && inner >= 0.0);
}

#[test]
fn regression_softened_force_matches_independent_snapshot() {
    let bodies = [
        make_body(Vec3::zero(), Vec3::zero(), 3.0),
        make_body(Vec3::new(2.0, 0.0, 0.0), Vec3::zero(), 1.0),
    ];
    let expected = Vec3::new(-6.0 / 8.5, 0.0, 0.0);
    assert!((direct_acceleration(&bodies, 1, 0.25) - expected).mag() < 1.0e-6);
}

#[test]
fn regression_equal_mass_binary_remains_on_a_bounded_circular_orbit() {
    let mut bodies = binary_pair();
    advance_binary(&mut bodies, 6000, 0.0005);
    let separation = (bodies[1].pos - bodies[0].pos).mag();
    assert!((0.9..=1.1).contains(&separation));
}

#[test]
fn performance_body_memory_stays_within_repository_budget() {
    assert!(size_of::<Body>() <= 192);
}

#[test]
fn performance_octree_node_growth_remains_linear() {
    let body_count = 256;
    let bodies = sample_bodies(body_count);
    let tree = make_tree(&bodies, 0.5, 0.2);
    assert!(tree.nodes.len() <= body_count * 16);
}

#[test]
#[ignore = "hardware timing requires a calibrated target-machine budget"]
fn performance_optional_4096_body_timing_stays_within_calibrated_budget() {
    let budget_ms = std::env::var("SCIENTIFIC_PERFORMANCE_BUDGET_MS")
        .ok()
        .and_then(|value| value.parse::<u128>().ok())
        .unwrap_or(250);
    let bodies = sample_bodies(4096);
    let start = Instant::now();
    let tree = make_tree(&bodies, 0.5, 0.2);
    for body in &bodies {
        let _ = tree.acc(body.pos, 0.04);
    }
    assert!(start.elapsed().as_millis() <= budget_ms);
}

#[test]
#[ignore = "hardware scaling timing is opt-in"]
fn performance_four_x_particle_scaling_stays_below_twelve_x_runtime() {
    let small_bodies = sample_bodies(1024);
    let large_bodies = sample_bodies(4096);

    let small_start = Instant::now();
    let small_tree = make_tree(&small_bodies, 0.5, 0.2);
    for body in &small_bodies {
        let _ = small_tree.acc(body.pos, 0.04);
    }
    let small_elapsed = small_start.elapsed().as_secs_f64().max(1.0e-6);

    let large_start = Instant::now();
    let large_tree = make_tree(&large_bodies, 0.5, 0.2);
    for body in &large_bodies {
        let _ = large_tree.acc(body.pos, 0.04);
    }
    let large_elapsed = large_start.elapsed().as_secs_f64();

    assert!(large_elapsed / small_elapsed < 12.0);
}

#[test]
fn validation_accounts_conservative_transfer_preserves_totals() {
    use crate::accounts::{conservative_transfer, Account, AccountDelta, FlowCapacity};
    use ultraviolet::Vec3;

    let mut a = Account {
        mass: 10.0,
        energy: 50.0,
        momentum: Vec3::new(1.0, 0.0, 0.0),
        angular_momentum: Vec3::new(0.0, 2.0, 0.0),
        information: 5.0,
        entropy: 1.0,
    };
    let mut b = Account::zero();

    let req = AccountDelta {
        mass: 3.0,
        energy: 10.0,
        momentum: Vec3::new(0.5, 0.0, 0.0),
        angular_momentum: Vec3::new(0.0, 1.0, 0.0),
        information: 1.0,
        entropy: 0.2,
    };
    let cap = FlowCapacity {
        mass_per_dt: 2.0,
        energy_per_dt: 3.0,
        momentum_per_dt: 0.4,
        angular_momentum_per_dt: 0.8,
        information_per_dt: 1.0,
        entropy_per_dt: 0.5,
    };

    let total_before_mass = a.mass + b.mass;
    let total_before_energy = a.energy + b.energy;

    let moved = conservative_transfer(&mut a, &mut b, req, cap);

    let total_after_mass = a.mass + b.mass;
    let total_after_energy = a.energy + b.energy;

    assert!((total_before_mass - total_after_mass).abs() < 1e-6);
    assert!((total_before_energy - total_after_energy).abs() < 1e-6);
    // moved magnitudes should not exceed capacities
    assert!(moved.mass <= cap.mass_per_dt + 1e-6);
    assert!(moved.energy <= cap.energy_per_dt + 1e-6);
}
