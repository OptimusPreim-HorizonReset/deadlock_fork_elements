#![allow(dead_code)]
use ultraviolet::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Account {
    pub mass: f32,
    pub energy: f32,
    pub momentum: Vec3,
    pub angular_momentum: Vec3,
    pub information: f32,
    pub entropy: f32,
}

impl Account {
    pub fn zero() -> Self {
        Self {
            mass: 0.0,
            energy: 0.0,
            momentum: Vec3::zero(),
            angular_momentum: Vec3::zero(),
            information: 0.0,
            entropy: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AccountDelta {
    pub mass: f32,
    pub energy: f32,
    pub momentum: Vec3,
    pub angular_momentum: Vec3,
    pub information: f32,
    pub entropy: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct FlowCapacity {
    pub mass_per_dt: f32,
    pub energy_per_dt: f32,
    pub momentum_per_dt: f32,
    pub angular_momentum_per_dt: f32,
    pub information_per_dt: f32,
    pub entropy_per_dt: f32,
}

/// Transfer requested amounts from `from` to `to`, clamped by the provided capacity and
/// by the available resources on `from`.
///
/// `requested` is interpreted as non-negative amounts to move from `from` -> `to`.
/// The function returns the actual amounts that were moved.
pub fn conservative_transfer(
    from: &mut Account,
    to: &mut Account,
    requested: AccountDelta,
    capacity: FlowCapacity,
) -> AccountDelta {
    // Scalars: mass, energy, information, entropy
    let mass_move = requested
        .mass
        .max(0.0)
        .min(capacity.mass_per_dt)
        .min(from.mass);
    from.mass -= mass_move;
    to.mass += mass_move;

    let energy_move = requested
        .energy
        .max(0.0)
        .min(capacity.energy_per_dt)
        .min(from.energy);
    from.energy -= energy_move;
    to.energy += energy_move;

    let info_move = requested
        .information
        .max(0.0)
        .min(capacity.information_per_dt)
        .min(from.information);
    from.information -= info_move;
    to.information += info_move;

    let entropy_move = requested
        .entropy
        .max(0.0)
        .min(capacity.entropy_per_dt)
        .min(from.entropy);
    from.entropy -= entropy_move;
    to.entropy += entropy_move;

    // Vector quantities: momentum, angular_momentum
    let mut momentum_move = Vec3::zero();
    let req_mom = requested.momentum;
    let req_mom_mag = req_mom.mag();
    if req_mom_mag > 0.0 {
        let allowed_mag = req_mom_mag.min(capacity.momentum_per_dt);
        let from_mom_mag = from.momentum.mag();
        let transfer_mag = allowed_mag.min(from_mom_mag);
        if transfer_mag > 0.0 {
            momentum_move = req_mom * (transfer_mag / req_mom_mag);
            from.momentum -= momentum_move;
            to.momentum += momentum_move;
        }
    }

    let mut angular_move = Vec3::zero();
    let req_ang = requested.angular_momentum;
    let req_ang_mag = req_ang.mag();
    if req_ang_mag > 0.0 {
        let allowed_ang = req_ang_mag.min(capacity.angular_momentum_per_dt);
        let from_ang_mag = from.angular_momentum.mag();
        let transfer_ang = allowed_ang.min(from_ang_mag);
        if transfer_ang > 0.0 {
            angular_move = req_ang * (transfer_ang / req_ang_mag);
            from.angular_momentum -= angular_move;
            to.angular_momentum += angular_move;
        }
    }

    AccountDelta {
        mass: mass_move,
        energy: energy_move,
        momentum: momentum_move,
        angular_momentum: angular_move,
        information: info_move,
        entropy: entropy_move,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ultraviolet::Vec3;

    #[test]
    fn scalar_transfer_is_clamped_by_capacity_and_availability() {
        let mut a = Account {
            mass: 10.0,
            energy: 5.0,
            momentum: Vec3::zero(),
            angular_momentum: Vec3::zero(),
            information: 2.0,
            entropy: 1.0,
        };
        let mut b = Account::zero();

        let req = AccountDelta {
            mass: 3.0,
            energy: 10.0,
            momentum: Vec3::zero(),
            angular_momentum: Vec3::zero(),
            information: 1.0,
            entropy: 0.5,
        };
        let cap = FlowCapacity {
            mass_per_dt: 2.0,
            energy_per_dt: 2.0,
            momentum_per_dt: 0.0,
            angular_momentum_per_dt: 0.0,
            information_per_dt: 1.0,
            entropy_per_dt: 1.0,
        };

        let moved = conservative_transfer(&mut a, &mut b, req, cap);
        assert_eq!(moved.mass, 2.0);
        assert_eq!(a.mass, 8.0);
        assert_eq!(b.mass, 2.0);

        // energy: requested 10 but capacity 2 and from.energy 5 -> allowed 2
        assert_eq!(moved.energy, 2.0);
        assert_eq!(a.energy, 3.0);
        assert_eq!(b.energy, 2.0);

        // information/entropy
        assert_eq!(moved.information, 1.0);
        assert_eq!(a.information, 1.0);
        assert_eq!(b.information, 1.0);

        assert_eq!(moved.entropy, 0.5);
        assert_eq!(a.entropy, 0.5);
        assert_eq!(b.entropy, 0.5);
    }

    #[test]
    fn vector_momentum_transfer_respects_direction_and_capacity() {
        let mut a = Account {
            mass: 5.0,
            energy: 0.0,
            momentum: Vec3::new(1.0, 0.0, 0.0),
            angular_momentum: Vec3::zero(),
            information: 0.0,
            entropy: 0.0,
        };
        let mut b = Account::zero();

        let req = AccountDelta {
            mass: 0.0,
            energy: 0.0,
            momentum: Vec3::new(0.5, 0.0, 0.0),
            angular_momentum: Vec3::zero(),
            information: 0.0,
            entropy: 0.0,
        };

        let cap = FlowCapacity {
            mass_per_dt: 1.0,
            energy_per_dt: 1.0,
            momentum_per_dt: 0.4, // capacity smaller than requested magnitude
            angular_momentum_per_dt: 1.0,
            information_per_dt: 1.0,
            entropy_per_dt: 1.0,
        };

        let moved = conservative_transfer(&mut a, &mut b, req, cap);
        // moved momentum magnitude should equal 0.4
        assert!((moved.momentum.mag() - 0.4).abs() < 1e-6);
        // a.momentum should be reduced by 0.4
        assert!((a.momentum.mag() - 0.6).abs() < 1e-6);
        // b.momentum should be 0.4
        assert!((b.momentum.mag() - 0.4).abs() < 1e-6);
    }

    #[test]
    fn conservative_transfer_preserves_totals_when_within_capacity() {
        let mut a = Account {
            mass: 10.0,
            energy: 20.0,
            momentum: Vec3::new(2.0, 0.0, 0.0),
            angular_momentum: Vec3::new(1.0, 0.0, 0.0),
            information: 1.0,
            entropy: 0.5,
        };
        let mut b = Account {
            mass: 3.0,
            energy: 5.0,
            momentum: Vec3::new(0.5, 0.0, 0.0),
            angular_momentum: Vec3::new(0.2, 0.0, 0.0),
            information: 0.0,
            entropy: 0.0,
        };

        let before_mass = a.mass + b.mass;
        let before_energy = a.energy + b.energy;
        let before_momentum = a.momentum + b.momentum;
        let before_ang = a.angular_momentum + b.angular_momentum;

        let req = AccountDelta {
            mass: 2.0,
            energy: 3.0,
            momentum: Vec3::new(0.5, 0.0, 0.0),
            angular_momentum: Vec3::new(0.5, 0.0, 0.0),
            information: 0.5,
            entropy: 0.1,
        };
        let cap = FlowCapacity {
            mass_per_dt: 10.0,
            energy_per_dt: 10.0,
            momentum_per_dt: 10.0,
            angular_momentum_per_dt: 10.0,
            information_per_dt: 10.0,
            entropy_per_dt: 10.0,
        };

        let _moved = conservative_transfer(&mut a, &mut b, req, cap);

        assert!((a.mass + b.mass - before_mass).abs() < 1e-6);
        assert!((a.energy + b.energy - before_energy).abs() < 1e-6);
        assert!(((a.momentum + b.momentum) - before_momentum).mag() < 1e-6);
        assert!(((a.angular_momentum + b.angular_momentum) - before_ang).mag() < 1e-6);
    }
}
