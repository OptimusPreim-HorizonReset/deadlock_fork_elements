#![allow(dead_code)]
use crate::accounts::Account;
use crate::body::Body;
use ultraviolet::Vec3;

#[derive(Clone)]
pub struct GalaxyState {
    pub id: u64,
    pub bodies: Vec<Body>,
    pub accounts: Account,
    pub equatorial_plane_normal: Vec3,
    pub equatorial_plane_center: Vec3,
}

impl GalaxyState {
    pub fn new(id: u64, bodies: Vec<Body>) -> Self {
        let mut total_mass = 0.0_f32;
        let mut momentum = Vec3::zero();
        let mut angular = Vec3::zero();
        for b in &bodies {
            total_mass += b.mass;
            momentum += b.vel * b.mass;
            // local angular momentum per body approximated as r x (m v)
            angular += b.pos.cross(b.vel) * b.mass;
        }
        let accounts = Account {
            mass: total_mass,
            energy: 0.0,
            momentum,
            angular_momentum: angular,
            information: 0.0,
            entropy: 0.0,
        };
        Self {
            id,
            bodies,
            accounts,
            equatorial_plane_normal: Vec3::new(0.0, 0.0, 1.0),
            equatorial_plane_center: Vec3::zero(),
        }
    }

    pub fn delegate_mass(&self) -> f32 {
        self.accounts.mass
    }

    pub fn delegate_momentum(&self) -> Vec3 {
        self.accounts.momentum
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ultraviolet::Vec3;

    #[test]
    fn galaxy_aggregates_mass_and_momentum() {
        let b1 = Body::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            2.0,
            0.4,
            0.0,
            Vec3::new(0.0, 0.0, 1.0),
        );
        let b2 = Body::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            3.0,
            0.4,
            0.0,
            Vec3::new(0.0, 0.0, 1.0),
        );
        let g = GalaxyState::new(1, vec![b1, b2]);
        assert!((g.delegate_mass() - 5.0).abs() < 1e-6);
        let mom = g.delegate_momentum();
        assert!((mom.x - (2.0 * 1.0 + 3.0 * 0.0)).abs() < 1e-6);
        assert!((mom.y - (2.0 * 0.0 + 3.0 * 2.0)).abs() < 1e-6);
    }
}
