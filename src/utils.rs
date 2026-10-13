#![allow(dead_code)]
use crate::{body::Body, galaxy_templates::GalaxyTemplate};
use ultraviolet::Vec2;

/// Creates a single spiral galaxy centered at the origin with stable Keplerian orbits.
pub fn uniform_disc(n: usize) -> Vec<Body> {
    fastrand::seed(0);
    GalaxyTemplate::spiral(n).generate(Vec2::zero(), Vec2::zero())
}
