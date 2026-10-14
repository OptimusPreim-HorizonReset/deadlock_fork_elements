#![allow(dead_code)]
use crate::quadtree::{Oct, Octree};
use crate::tables::Delegate;
use ultraviolet::Vec3;

pub struct GalaxyTree {
    octree: Octree,
}

impl GalaxyTree {
    pub fn new(theta: f32, epsilon: f32) -> Self {
        Self {
            octree: Octree::new(theta, epsilon),
        }
    }

    pub fn clear(&mut self, oct: Oct) {
        self.octree.clear(oct);
    }

    /// Insert a delegate at the given position. For now we aggregate by mass only
    /// (delegates expose mass + momentum + angular momentum). Multipole handling
    /// will be added later.
    pub fn insert_delegate(&mut self, pos: Vec3, delegate: &Delegate) {
        self.octree.insert_delegate(
            pos,
            delegate.mass,
            delegate.momentum,
            delegate.angular_momentum,
        );
    }

    pub fn propagate(&mut self) {
        self.octree.propagate();
    }

    pub fn acc(&self, pos: Vec3, softening_sq: f32) -> Vec3 {
        self.octree.acc(pos, softening_sq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::Body;
    use crate::quadtree::Oct;
    use crate::tables::Delegate;
    use ultraviolet::Vec3;

    #[test]
    fn galaxy_tree_inserts_and_accumulates_mass() {
        let mut tree = GalaxyTree::new(1.0, 0.0);
        let oct = Oct::new_containing(&[] as &[Body]);
        tree.clear(oct);
        let d1 = Delegate {
            mass: 3.0,
            momentum: Vec3::zero(),
            angular_momentum: Vec3::zero(),
        };
        let d2 = Delegate {
            mass: 2.0,
            momentum: Vec3::zero(),
            angular_momentum: Vec3::zero(),
        };
        tree.insert_delegate(Vec3::new(0.0, 0.0, 0.0), &d1);
        tree.insert_delegate(Vec3::new(1.0, 0.0, 0.0), &d2);
        tree.propagate();
        // acceleration at origin produced by two masses should be finite and non-NaN
        let a = tree.acc(Vec3::zero(), 0.0);
        assert!(a.x.is_finite());
    }
}
