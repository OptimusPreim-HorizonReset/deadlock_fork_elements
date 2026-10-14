#[derive(Clone, Copy, Debug)]
pub struct GasBillboard {
    density: f32,
    temperature: f32,
}

impl GasBillboard {
    pub fn new(density: f32, temperature: f32) -> Self {
        Self {
            density: if density.is_finite() {
                density.max(0.0)
            } else {
                0.0
            },
            temperature: if temperature.is_finite() {
                temperature.max(0.0)
            } else {
                0.0
            },
        }
    }

    pub fn alpha_layers(self) -> [f32; 3] {
        let density = (self.density / (1.0 + self.density)).clamp(0.0, 1.0);
        let thermal = (self.temperature / (1.0 + self.temperature)).clamp(0.0, 1.0);
        let outer = (density * 0.9).clamp(0.0, 0.9);
        [
            outer,
            (outer * (0.65 + thermal * 0.05)).clamp(0.0, outer),
            (outer * (0.3 + thermal * 0.1)).clamp(0.0, outer),
        ]
    }

    pub fn color(self) -> [u8; 3] {
        let thermal = (self.temperature / (1.0 + self.temperature)).clamp(0.0, 1.0);
        [
            (80.0 + 160.0 * thermal).round() as u8,
            (190.0 - 85.0 * thermal).round() as u8,
            (255.0 - 35.0 * thermal).round() as u8,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::GasBillboard;

    #[test]
    fn alpha_layers_are_bounded_and_outer_to_inner_ordered() {
        for density in [0.0, 0.1, 1.0, 1000.0] {
            for temperature in [0.0, 0.5, 10.0, 1000.0] {
                let [outer, middle, inner] = GasBillboard::new(density, temperature).alpha_layers();
                assert!(outer.is_finite() && (0.0..=0.9).contains(&outer));
                assert!(middle.is_finite() && (0.0..=outer).contains(&middle));
                assert!(inner.is_finite() && (0.0..=middle).contains(&inner));
            }
        }
    }

    #[test]
    fn invalid_gas_visual_inputs_are_sanitized() {
        let billboard = GasBillboard::new(f32::NAN, f32::INFINITY);
        assert_eq!(billboard.alpha_layers(), [0.0; 3]);
        assert_ne!(
            GasBillboard::new(1.0, 0.0).color(),
            GasBillboard::new(1.0, 10.0).color()
        );
    }
}
