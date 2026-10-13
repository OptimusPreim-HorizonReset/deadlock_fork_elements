use fastrand;

// Deterministic seed helpers and simple astrophysical types used by the simulation.
// This module intentionally contains small, well-tested helpers and plain data
// structures; heavy algorithms (hydro, fragmentation, SN) will be added incrementally
// in simulation.rs and call into these helpers.

/// Simple 64-bit mix for deterministic seeding.
pub fn mix64(a: u64, b: u64, c: u64) -> u64 {
    let mut x = a.wrapping_add(0x9e3779b97f4a7c15);
    x ^= b.wrapping_mul(0xbf58476d1ce4e5b9);
    x = x.wrapping_mul(0x94d049bb133111eb);
    x ^ c
}

/// Region seed built from integer cell coords, frame, and a global seed to guarantee
/// deterministic behavior for local decisions (spawning, sampling, etc.).
pub fn region_seed(xi: i64, yi: i64, zi: i64, frame: u64, global_seed: u64) -> u64 {
    mix64(global_seed, xi as u64 ^ ((yi as u64) << 21), zi as u64 ^ frame)
}

/// Sample a power-law distributed mass deterministically from a provided RNG.
/// alpha is the Salpeter-like slope (e.g. 2.35). m_min and m_max are inclusive bounds.
pub fn sample_powerlaw_mass(rng: &mut fastrand::Rng, m_min: f32, m_max: f32, alpha: f32) -> f32 {
    let u = rng.f32();
    if (alpha - 1.0).abs() < 1e-6 {
        // special case: 1/x -> uniform in log-space
        let x = (m_min.ln() + u * (m_max.ln() - m_min.ln())).exp();
        x
    } else {
        let ia = 1.0 - alpha;
        let mminp = m_min.powf(ia);
        let mmaxp = m_max.powf(ia);
        (u * (mmaxp - mminp) + mminp).powf(1.0 / ia)
    }
}

/// Simple gas cell state used as a light-weight abstraction for hydrodynamic
/// collapse/fragmentation decisions. The simulation still uses Body.gas_density etc.,
/// but this type centralizes extra fields for future use.
#[derive(Clone, Debug, Default)]
pub struct GasState {
    pub mass: f32,
    pub internal_energy: f32,
    pub metallicity: f32,
    pub smoothing_h: f32,
    pub last_update_frame: usize,
}

/// Star lifecycle stages used by the simple deterministic star evolution state machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StarStage {
    Protostar,
    MainSequence,
    RedGiant,
    PostAGB,
    RemnantNeutron,
    RemnantBlackHole,
}

impl Default for StarStage {
    fn default() -> Self {
        StarStage::Protostar
    }
}

/// Minimal deterministic star state used to track fuel, age and decide end-of-life events.
#[derive(Clone, Copy, Debug, Default)]
pub struct StarState {
    pub mass: f32,
    pub age: f32,
    pub stage: StarStage,
    pub fuel: f32,
    pub core_mass: f32,
    pub metallicity: f32,
    pub luminosity: f32,
}

impl StarState {
    pub fn new(mass: f32, metallicity: f32) -> Self {
        // Very simple fuel heuristic: fuel ∝ mass
        let fuel = (mass).max(0.0001);
        // rough luminosity scaling placeholder (L ∝ M^3)
        let luminosity = mass.powf(3.0).max(1e-6);
        StarState {
            mass,
            age: 0.0,
            stage: StarStage::Protostar,
            fuel,
            core_mass: mass * 0.1,
            metallicity,
            luminosity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_seed_is_deterministic() {
        let s1 = region_seed(10, -3, 7, 123, 0xDEADBEEF12345678);
        let s2 = region_seed(10, -3, 7, 123, 0xDEADBEEF12345678);
        assert_eq!(s1, s2);
    }

    #[test]
    fn sample_powerlaw_is_deterministic_for_fixed_seed() {
        let seed = 0xFEEDFACECAFEBABEu64;
        let mut rng1 = fastrand::Rng::with_seed(seed);
        let mut rng2 = fastrand::Rng::with_seed(seed);
        let m1 = sample_powerlaw_mass(&mut rng1, 0.1, 50.0, 2.35);
        let m2 = sample_powerlaw_mass(&mut rng2, 0.1, 50.0, 2.35);
        assert!((m1 - m2).abs() < 1e-6);
    }
}
