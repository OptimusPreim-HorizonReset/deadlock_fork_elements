#![allow(dead_code)]

use crate::accounts::FlowCapacity;
use crate::galaxy::GalaxyState;
use crate::tables::Table;
use ultraviolet::Vec3;

/// Deterministic institution emergence rule (v1).
///
/// Heuristic (v1): if the galaxy's total mass exceeds a threshold relative to
/// config.central_mass, instantiate a single galaxy-level Table representing a
/// coordinating institution. This is intentionally simple and deterministic
/// for initial validation; future versions may compose multiple tables or use
/// more sophisticated criteria.
#[allow(dead_code)]
pub fn emergence_v1(
    g: &GalaxyState,
    config: &crate::config::InformationsConfig,
    start_id: u64,
) -> Vec<Table> {
    let mut out = Vec::new();
    // Threshold: create an institution when galaxy mass is > 10x central_mass
    let threshold = config.central_mass * 10.0_f32;
    if g.accounts.mass > threshold {
        let capacity = FlowCapacity {
            mass_per_dt: config.flow_capacity_mass_per_dt,
            energy_per_dt: config.flow_capacity_energy_per_dt,
            momentum_per_dt: config.flow_capacity_momentum_per_dt,
            angular_momentum_per_dt: config.flow_capacity_angular_momentum_per_dt,
            information_per_dt: 1.0,
            entropy_per_dt: 1.0,
        };
        // Use the galaxy delegate as the initial account state for the emergent table.
        let accounts = g.accounts;
        let mut table = Table::new(start_id, accounts, capacity, config.timescale_galaxy);

        // Compute a simple center-of-mass position for the emergent table when possible
        // so renderers and organigram views can place it in world-space.
        if accounts.mass > 0.0 {
            let mut center = Vec3::zero();
            for b in &g.bodies {
                center += b.pos * b.mass;
            }
            center /= accounts.mass;
            table.position = Some(center);
        }

        // If the flow-dynamics feature is enabled, register the emergent table globally
        // so the renderer and other debug tools can visualize it. Replace existing entry
        // if one with the same id already exists to avoid duplicate registrations.
        if config.enable_flow_dynamics_model {
            let mut reg = crate::tables::TABLES.lock();
            if let Some(idx) = reg.iter().position(|t| t.id == table.id) {
                reg[idx] = table.clone();
            } else {
                reg.push(table.clone());
            }
        }
        out.push(table);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ultraviolet::Vec3;

    #[test]
    fn emergence_v1_creates_table_when_threshold_exceeded() {
        // Build two bodies summing to a large mass
        let b1 = crate::body::Body::new(
            Vec3::zero(),
            Vec3::zero(),
            2e7,
            0.4,
            0.0,
            Vec3::new(0.0, 0.0, 1.0),
        );
        let g = GalaxyState::new(1, vec![b1]);
        let cfg = crate::config::InformationsConfig::default();
        // default central_mass is large (1e6) so 2e7 > 10*1e6 -> table should be created
        let tables = emergence_v1(&g, &cfg, 100);
        assert!(!tables.is_empty());
        assert_eq!(tables[0].id, 100);
        assert!((tables[0].accounts.mass - g.accounts.mass).abs() < 1e-6);
    }

    #[test]
    fn emergence_v1_no_table_when_below_threshold() {
        let b1 = crate::body::Body::new(
            Vec3::zero(),
            Vec3::zero(),
            1.0,
            0.4,
            0.0,
            Vec3::new(0.0, 0.0, 1.0),
        );
        let g = GalaxyState::new(2, vec![b1]);
        let cfg = crate::config::InformationsConfig::default();
        let tables = emergence_v1(&g, &cfg, 200);
        assert!(tables.is_empty());
    }

    #[test]
    fn emergence_v1_registers_table_in_registry_when_enabled() {
        // Ensure TABLES registry is cleared for the test
        crate::tables::TABLES.lock().clear();

        let b1 = crate::body::Body::new(
            Vec3::zero(),
            Vec3::zero(),
            2e7,
            0.4,
            0.0,
            Vec3::new(0.0, 0.0, 1.0),
        );
        let g = GalaxyState::new(3, vec![b1]);
        let cfg = crate::config::InformationsConfig {
            enable_flow_dynamics_model: true, // enable registry behavior
            ..Default::default()
        };
        let tables = emergence_v1(&g, &cfg, 999);
        assert!(!tables.is_empty());
        let reg = crate::tables::TABLES.lock();
        assert!(reg.iter().any(|t| t.id == 999));
        // cleanup
        drop(reg);
        crate::tables::TABLES.lock().clear();
    }
}
