#![allow(dead_code)]
use crate::accounts::{conservative_transfer, AccountDelta, FlowCapacity};
use crate::galaxy::GalaxyState;
use ultraviolet::Vec3;

pub struct NegotiationResult {
    pub moved: AccountDelta,
}

/// Simple contact parliament negotiation between two galaxy states.
/// This implementation is intentionally conservative and lightweight for M6:
/// it attempts a small transfer of angular momentum from the more-rotating
/// galaxy to the less-rotating one, clamped by the provided capacity.
pub fn negotiate_between(
    a: &mut GalaxyState,
    b: &mut GalaxyState,
    capacity: FlowCapacity,
) -> NegotiationResult {
    // Compute angular magnitudes
    let a_ang = a.accounts.angular_momentum;
    let b_ang = b.accounts.angular_momentum;
    let a_mag = a_ang.mag();
    let b_mag = b_ang.mag();

    // If there's no angular momentum difference, do nothing
    if (a_mag - b_mag).abs() < 1e-8 {
        return NegotiationResult {
            moved: AccountDelta::default(),
        };
    }

    // Decide direction: from higher to lower
    let (from, to, from_ang) = if a_mag > b_mag {
        (a, b, a_ang)
    } else {
        (b, a, b_ang)
    };

    // Request a small fraction of the donor's angular momentum (e.g., 5%)
    let request_fraction = 0.05_f32;
    let requested_ang = from_ang * request_fraction;

    let requested = AccountDelta {
        mass: 0.0,
        energy: 0.0,
        momentum: Vec3::zero(),
        angular_momentum: requested_ang,
        information: 0.0,
        entropy: 0.0,
    };

    let moved = conservative_transfer(&mut from.accounts, &mut to.accounts, requested, capacity);

    NegotiationResult { moved }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ultraviolet::Vec3;

    fn make_galaxy_with_ang(ang: Vec3) -> crate::galaxy::GalaxyState {
        // construct a minimal galaxystate with zero bodies but a provided angular momentum
        let mut g = crate::galaxy::GalaxyState::new(0, vec![]);
        g.accounts.angular_momentum = ang;
        g
    }

    #[test]
    fn negotiation_preserves_total_angular_momentum() {
        let mut g1 = make_galaxy_with_ang(Vec3::new(10.0, 0.0, 0.0));
        let mut g2 = make_galaxy_with_ang(Vec3::new(2.0, 0.0, 0.0));

        let total_before = g1.accounts.angular_momentum + g2.accounts.angular_momentum;

        let cap = FlowCapacity {
            mass_per_dt: 1.0,
            energy_per_dt: 1.0,
            momentum_per_dt: 1.0,
            angular_momentum_per_dt: 1.0, // small cap to force partial transfer
            information_per_dt: 1.0,
            entropy_per_dt: 1.0,
        };

        let res = negotiate_between(&mut g1, &mut g2, cap);
        let total_after = g1.accounts.angular_momentum + g2.accounts.angular_momentum;

        assert!((total_before - total_after).mag() < 1e-6);
        // moved angular magnitude should not be negative
        assert!(res.moved.angular_momentum.mag() >= 0.0);
    }
}
