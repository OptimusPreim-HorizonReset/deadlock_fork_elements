#![allow(dead_code, clippy::upper_case_acronyms, clippy::manual_is_multiple_of)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimescaleTier {
    SMBH,
    StarFormation,
    Galaxy,
    Cluster,
}

impl TimescaleTier {
    /// Return a default update interval (frames) for each tier. These defaults match
    /// the conceptual document but can be overridden in configuration.
    pub fn default_interval(&self) -> usize {
        match self {
            TimescaleTier::SMBH => 1,
            TimescaleTier::StarFormation => 100,
            TimescaleTier::Galaxy => 10_000,
            TimescaleTier::Cluster => 1_000_000,
        }
    }
}

/// Simple scheduler helper that answers whether a given frame should update a tier.
#[derive(Clone, Copy, Debug)]
pub struct TimescaleScheduler {
    pub frame: usize,
}

impl TimescaleScheduler {
    pub fn new() -> Self {
        Self { frame: 0 }
    }

    /// Advance the global frame counter and return true if the tier should be updated
    /// at this new frame. If `interval` is 0, the tier is considered disabled.
    pub fn tick_and_should_update(&mut self, interval: usize) -> bool {
        self.frame = self.frame.wrapping_add(1);
        if interval == 0 {
            return false;
        }
        self.frame % interval == 0
    }

    /// Advance the global frame counter by one without querying any interval.
    pub fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
    }

    /// Query whether the given interval (in frames) should be updated at the
    /// current frame WITHOUT advancing the internal counter. If `interval` is 0,
    /// the tier is considered disabled.
    pub fn should_update(&self, interval: usize) -> bool {
        if interval == 0 {
            return false;
        }
        self.frame % interval == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_intervals_are_reasonable() {
        assert_eq!(TimescaleTier::SMBH.default_interval(), 1);
        assert_eq!(TimescaleTier::StarFormation.default_interval(), 100);
        assert_eq!(TimescaleTier::Galaxy.default_interval(), 10_000);
        assert_eq!(TimescaleTier::Cluster.default_interval(), 1_000_000);
    }

    #[test]
    fn scheduler_ticks_as_expected() {
        let mut s = TimescaleScheduler::new();
        // interval 2 -> should update every 2 frames
        assert!(!s.tick_and_should_update(2)); // frame 1
        assert!(s.tick_and_should_update(2)); // frame 2
        assert!(!s.tick_and_should_update(2)); // frame 3
        assert!(s.tick_and_should_update(2)); // frame 4
    }
}
