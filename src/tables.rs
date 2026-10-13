#![allow(dead_code)]
use crate::accounts::{conservative_transfer, Account, AccountDelta, FlowCapacity};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use ultraviolet::Vec3;

#[derive(Clone, Debug)]
pub struct Constitution {
    /// Hard rules are enforced by the transfer primitives and never disabled.
    /// In this simplified model this struct is a placeholder for future
    /// operative rules.
    pub enforce_conservation: bool,
}

impl Default for Constitution {
    fn default() -> Self {
        Self {
            enforce_conservation: true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Delegate {
    pub mass: f32,
    pub momentum: Vec3,
    pub angular_momentum: Vec3,
}

#[derive(Clone, Debug)]
pub struct Table {
    pub id: u64,
    pub accounts: Account,
    pub capacity: FlowCapacity,
    pub constitution: Constitution,
    pub children: Vec<u64>,
    pub parent: Option<u64>,
    /// Optional spatial position in world coordinates (used by renderer/organigram)
    pub position: Option<Vec3>,
    /// timescale expressed as update interval in frames (e.g., SMBH=1, star=100)
    pub timescale: usize,
}

/// Global registry for emergent tables used by the renderer & debug tools.
/// This is intentionally lightweight and intended for visualization/debugging only.
pub static TABLES: Lazy<Mutex<Vec<Table>>> = Lazy::new(|| Mutex::new(Vec::new()));

impl Table {
    pub fn new(id: u64, accounts: Account, capacity: FlowCapacity, timescale: usize) -> Self {
        Self {
            id,
            accounts,
            capacity,
            constitution: Constitution::default(),
            children: Vec::new(),
            parent: None,
            position: None,
            timescale,
        }
    }

    /// Export a compact delegate describing the table's outward state
    pub fn delegate(&self) -> Delegate {
        Delegate {
            mass: self.accounts.mass,
            momentum: self.accounts.momentum,
            angular_momentum: self.accounts.angular_momentum,
        }
    }

    /// Attempt to transfer requested deltas from this table to another table.
    /// Returns the actual moved amounts as an AccountDelta.
    pub fn transfer_to(&mut self, other: &mut Table, requested: AccountDelta) -> AccountDelta {
        conservative_transfer(
            &mut self.accounts,
            &mut other.accounts,
            requested,
            self.capacity,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::Account;
    use ultraviolet::Vec3;

    #[test]
    fn delegate_reflects_accounts() {
        let acc = Account {
            mass: 42.0,
            energy: 1.0,
            momentum: Vec3::new(1.0, 2.0, 3.0),
            angular_momentum: Vec3::new(0.1, 0.2, 0.3),
            information: 0.0,
            entropy: 0.0,
        };
        let cap = FlowCapacity {
            mass_per_dt: 10.0,
            energy_per_dt: 10.0,
            momentum_per_dt: 5.0,
            angular_momentum_per_dt: 1.0,
            information_per_dt: 1.0,
            entropy_per_dt: 1.0,
        };
        let t = Table::new(1, acc, cap, 1000);
        let d = t.delegate();
        assert!((d.mass - 42.0).abs() < 1e-6);
        assert!((d.momentum.x - 1.0).abs() < 1e-6);
    }

    #[test]
    fn transfer_between_tables_uses_capacity() {
        let a_acc = Account {
            mass: 10.0,
            energy: 5.0,
            momentum: Vec3::new(1.0, 0.0, 0.0),
            angular_momentum: Vec3::zero(),
            information: 1.0,
            entropy: 0.5,
        };
        let b_acc = Account::zero();
        let cap = FlowCapacity {
            mass_per_dt: 2.0,
            energy_per_dt: 2.0,
            momentum_per_dt: 0.5,
            angular_momentum_per_dt: 1.0,
            information_per_dt: 1.0,
            entropy_per_dt: 1.0,
        };
        let mut t1 = Table::new(1, a_acc, cap, 100);
        let mut t2 = Table::new(2, b_acc, cap, 100);
        let req = AccountDelta {
            mass: 5.0,
            energy: 5.0,
            momentum: Vec3::new(1.0, 0.0, 0.0),
            angular_momentum: Vec3::zero(),
            information: 1.0,
            entropy: 0.5,
        };
        let moved = t1.transfer_to(&mut t2, req);
        // mass move limited by capacity to 2.0
        assert_eq!(moved.mass, 2.0);
        assert!((t1.accounts.mass - 8.0).abs() < 1e-6);
        assert!((t2.accounts.mass - 2.0).abs() < 1e-6);
    }
}
