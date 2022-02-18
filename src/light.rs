use crate::config::*;
use crate::common::{Ray};

use rand::prelude::*;

pub trait Light {
    fn emit(&self, _rng: &mut ThreadRng) -> (Ray, Color3f);
}

pub struct SphereLight {
    c: Point3f,
    r: Fp,
    e: Color3f,
}

impl SphereLight {
    pub fn new(center: Point3f, radius: Fp, e: Color3f) -> Self {
        Self {
            c: center,
            r: radius,
            e
        }
    }
}

impl Light for SphereLight {
    fn emit(&self, rng: &mut ThreadRng) -> (Ray, Color3f) {
        let r1: Fp = rng.gen();
        let r2: Fp = rng.gen();
        let phi = 2. * consts::PI * r1;
        let cos_theta = 1. - 2. * r2;
        let sin_theta = 2. * (r2 * (1. - r2)).sqrt();

        let x = phi.cos() * sin_theta;
        let y = phi.sin() * sin_theta;
        let z = cos_theta;
        let d = Vector3f::new(x, y, z);
        let ray = Ray::new(self.c + self.r * d, d);
        (ray, self.e)
    }
} 
