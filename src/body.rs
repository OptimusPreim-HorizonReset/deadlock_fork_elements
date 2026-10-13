use ultraviolet::Vec3;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodySegment {
    Unspecified,
    Core,
    Bulge,
    Orbital,
    Gas,
    Satellite,
    ManualSpawn,
    Accretion,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct Body {
    pub pos: Vec3,
    pub vel: Vec3,
    pub acc: Vec3,
    pub mass: f32,
    pub base_radius: f32,
    pub equatorial_radius: f32,
    pub polar_radius: f32,
    pub rotation_axis: Vec3,
    pub angular_speed: f32,
    pub spin_angle: f32,
    pub body_id: u64,
    pub element_atomic_number: Option<u8>,
    pub molecule_id: Option<u64>,
    // Optional astrophysical star state (protostar -> main sequence -> remnant)
    pub star_state: Option<crate::astro::StarState>,
    pub segment: BodySegment,
    pub gas_density: f32,
    pub gas_temperature: f32,
    // Role weights determine probabilistic behavioural leaning (sum to 1.0)
    pub roles: crate::roles::RoleWeights,
    // Local angular momentum and magnetization used by role/rules
    pub local_angular_momentum: Vec3,
    pub magnetization: f32,
    // Bias controlling accretion vs ejection tendency (range may be normalized elsewhere)
    pub accretion_ejection_bias: f32,
}

impl Body {
    fn sanitize_vec(v: Vec3) -> Vec3 {
        Vec3::new(
            if v.x.is_finite() { v.x } else { 0.0 },
            if v.y.is_finite() { v.y } else { 0.0 },
            if v.z.is_finite() { v.z } else { 0.0 },
        )
    }

    pub fn new(
        pos: Vec3,
        vel: Vec3,
        mass: f32,
        radius: f32,
        angular_speed: f32,
        rotation_axis: Vec3,
    ) -> Self {
        let rotation_axis = if rotation_axis == Vec3::zero() {
            Vec3::new(0.0, 0.0, 1.0)
        } else {
            rotation_axis.normalized()
        };

        let scaled_radius = radius * 0.25;
        let mut body = Self {
            pos: Self::sanitize_vec(pos),
            vel: Self::sanitize_vec(vel),
            acc: Vec3::zero(),
            mass: if mass.is_finite() && mass > 0.0 {
                mass
            } else {
                1.0
            },
            base_radius: if scaled_radius.is_finite() && scaled_radius > 0.0 {
                scaled_radius
            } else {
                1.0
            },
            equatorial_radius: if scaled_radius.is_finite() && scaled_radius > 0.0 {
                scaled_radius
            } else {
                1.0
            },
            polar_radius: if scaled_radius.is_finite() && scaled_radius > 0.0 {
                scaled_radius
            } else {
                1.0
            },
            rotation_axis,
            angular_speed: if angular_speed.is_finite() {
                angular_speed
            } else {
                0.0
            },
            spin_angle: 0.0,
            body_id: 0,
            element_atomic_number: None,
            molecule_id: None,
            star_state: None,
            segment: BodySegment::Unspecified,
            gas_density: 0.0,
            gas_temperature: 0.0,
            // default role weights: passive reservoir
            roles: crate::roles::RoleWeights::default(),
            local_angular_momentum: Vec3::zero(),
            magnetization: 0.0,
            accretion_ejection_bias: 0.0,
        };
        body.update_shape();
        body
    }

    pub fn with_element(mut self, atomic_number: u8, segment: BodySegment) -> Self {
        if (1..=118).contains(&atomic_number) {
            self.element_atomic_number = Some(atomic_number);
            self.segment = segment;
        }
        self
    }

    pub fn with_id(mut self, body_id: u64) -> Self {
        self.body_id = body_id;
        self
    }

    pub fn with_gas_state(mut self, density: f32, temperature: f32) -> Self {
        self.segment = BodySegment::Gas;
        self.gas_density = if density.is_finite() {
            density.max(0.0)
        } else {
            0.0
        };
        self.gas_temperature = if temperature.is_finite() {
            temperature.max(0.0)
        } else {
            0.0
        };
        self
    }

    pub fn update(&mut self, dt: f32) {
        self.spin_angle += self.angular_speed * dt;
        self.vel += self.acc * dt;
        self.pos += self.vel * dt;

        self.spin_angle = if self.spin_angle.is_finite() {
            self.spin_angle
        } else {
            0.0
        };
        self.acc = Self::sanitize_vec(self.acc);
        self.vel = Self::sanitize_vec(self.vel);
        self.pos = Self::sanitize_vec(self.pos);

        self.update_shape();
    }

    pub fn projected_radius(&self) -> f32 {
        let depth_scale = (1.0 - self.pos.z * 0.002).clamp(0.55, 1.4);
        self.equatorial_radius * depth_scale
    }

    pub fn effective_radius(&self) -> f32 {
        (self.equatorial_radius * 2.0 + self.polar_radius) / 3.0
    }

    fn update_shape(&mut self) {
        let spin = self.angular_speed.abs();
        let axis_scale = self.rotation_axis.mag_sq().max(1.0);
        let equatorial_growth = 0.18 * spin * spin * axis_scale;
        let polar_flattening = 0.14 * spin * spin * axis_scale;

        self.equatorial_radius = self.base_radius * (1.0 + equatorial_growth).max(1.0);
        self.polar_radius =
            (self.base_radius * (1.0 - polar_flattening)).max(self.base_radius * 0.35);
    }
}
