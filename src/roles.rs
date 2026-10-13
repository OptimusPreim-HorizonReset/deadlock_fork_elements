#![allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct RoleWeights {
    pub accretor: f32,
    pub transporter: f32,
    pub heater: f32,
    pub densifier: f32,
    pub wind: f32,
    pub jet: f32,
    pub escapee: f32,
    pub reservoir: f32,
}

impl Default for RoleWeights {
    fn default() -> Self {
        // Default to passive/reservoir if nothing else is set.
        Self {
            accretor: 0.0,
            transporter: 0.0,
            heater: 0.0,
            densifier: 0.0,
            wind: 0.0,
            jet: 0.0,
            escapee: 0.0,
            reservoir: 1.0,
        }
    }
}

impl RoleWeights {
    /// Normalize the role weights so they sum to 1.0. If the sum is zero or NaN,
    /// resets to the default passive reservoir state.
    pub fn normalize(&mut self) {
        let sum = self.accretor
            + self.transporter
            + self.heater
            + self.densifier
            + self.wind
            + self.jet
            + self.escapee
            + self.reservoir;
        if !sum.is_finite() || sum <= 0.0 {
            *self = RoleWeights::default();
            return;
        }
        self.accretor /= sum;
        self.transporter /= sum;
        self.heater /= sum;
        self.densifier /= sum;
        self.wind /= sum;
        self.jet /= sum;
        self.escapee /= sum;
        self.reservoir /= sum;
    }

    /// Ensure all weights are finite and non-negative, then normalize.
    pub fn sanitize_and_normalize(&mut self) {
        self.accretor = if self.accretor.is_finite() && self.accretor > 0.0 {
            self.accretor
        } else {
            0.0
        };
        self.transporter = if self.transporter.is_finite() && self.transporter > 0.0 {
            self.transporter
        } else {
            0.0
        };
        self.heater = if self.heater.is_finite() && self.heater > 0.0 {
            self.heater
        } else {
            0.0
        };
        self.densifier = if self.densifier.is_finite() && self.densifier > 0.0 {
            self.densifier
        } else {
            0.0
        };
        self.wind = if self.wind.is_finite() && self.wind > 0.0 {
            self.wind
        } else {
            0.0
        };
        self.jet = if self.jet.is_finite() && self.jet > 0.0 {
            self.jet
        } else {
            0.0
        };
        self.escapee = if self.escapee.is_finite() && self.escapee > 0.0 {
            self.escapee
        } else {
            0.0
        };
        self.reservoir = if self.reservoir.is_finite() && self.reservoir > 0.0 {
            self.reservoir
        } else {
            0.0
        };
        self.normalize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_reservoir() {
        let r = RoleWeights::default();
        assert_eq!(r.reservoir, 1.0);
        assert!(r.accretor == 0.0 && r.wind == 0.0);
    }

    #[test]
    fn normalize_works() {
        let mut r = RoleWeights {
            accretor: 2.0,
            transporter: 1.0,
            heater: 0.0,
            densifier: 0.0,
            wind: 1.0,
            jet: 0.0,
            escapee: 0.0,
            reservoir: 0.0,
        };
        r.normalize();
        let sum = r.accretor
            + r.transporter
            + r.heater
            + r.densifier
            + r.wind
            + r.jet
            + r.escapee
            + r.reservoir;
        assert!((sum - 1.0).abs() < 1e-6);
        assert!((r.accretor - 0.5).abs() < 1e-6);
    }
}
