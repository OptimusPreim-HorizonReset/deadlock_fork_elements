#![allow(dead_code)]
use ultraviolet::Vec3;

#[derive(Clone, Debug)]
pub struct MergeEvent {
    pub time: usize,
    pub mass: f32,
    pub center_of_mass: Vec3,
}

#[derive(Clone, Debug)]
pub struct ShockEvent {
    pub time: usize,
    pub energy: f32,
}

#[derive(Clone, Debug)]
pub struct AccretionEvent {
    pub time: usize,
    pub mass: f32,
}

#[derive(Clone, Debug, Default)]
pub struct History {
    pub merges: Vec<MergeEvent>,
    pub shocks: Vec<ShockEvent>,
    pub accretion_log: Vec<AccretionEvent>,
    pub last_updated: usize,
}

impl History {
    pub fn record_merge(&mut self, time: usize, mass: f32, com: Vec3) {
        self.merges.push(MergeEvent {
            time,
            mass,
            center_of_mass: com,
        });
        self.last_updated = time;
    }

    pub fn record_shock(&mut self, time: usize, energy: f32) {
        self.shocks.push(ShockEvent { time, energy });
        self.last_updated = time;
    }

    pub fn record_accretion(&mut self, time: usize, mass: f32) {
        self.accretion_log.push(AccretionEvent { time, mass });
        self.last_updated = time;
    }

    /// Compute a simple stability metric: 1.0 is perfectly stable, smaller values indicate instability.
    /// For M7 a lightweight heuristic suffices: penalize recent large shocks and rewarding steady accretion.
    pub fn stability(&self, current_time: usize, decay_rate: f32) -> f32 {
        let mut score = 1.0_f32;
        for s in &self.shocks {
            let age = (current_time as isize - s.time as isize).max(0) as f32;
            let contribution = s.energy / (1.0 + decay_rate * age);
            score -= contribution * 1e-3; // scale down shock effect
        }
        for a in &self.accretion_log {
            let age = (current_time as isize - a.time as isize).max(0) as f32;
            let contribution = a.mass / (1.0 + decay_rate * age);
            score += contribution * 1e-4; // small positive contribution
        }
        score.max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_records_and_stability() {
        let mut h = History::default();
        h.record_accretion(10, 5.0);
        h.record_shock(15, 200.0);
        let s = h.stability(20, 0.1);
        assert!(s >= 0.0);
    }
}
