use crate::config::{Fp, Vector3f};
use crate::common::{Ray, HitRecord};

pub trait Object {
    fn intersect(&self, rec: &mut HitRecord, ray: &Ray) -> bool;
}

pub struct Sphere {
    c: Vector3f,
    r: Fp,    
}

impl Sphere {
    pub fn new(center: Vector3f, radius: Fp) -> Sphere {
        Sphere {
            c: center,
            r: radius
        }
    }
}

impl Object for Sphere {
    fn intersect(&self, rec: &mut HitRecord, ray: &Ray) -> bool {
        let oc = ray.o - self.c;
        let a = ray.d.norm_squared();
        let half_b = oc.dot(&ray.d);
        let c = oc.norm_squared() - self.r * self.r;

        let delta = half_b * half_b - a * c;
        if delta < 0.0 {
            return false
        }

        let sqrtd = delta.sqrt();
        let t = (-half_b - sqrtd) / a;
        rec.t = t;
        rec.pos = ray.at(t);
        rec.norm = (rec.pos - self.c) / self.r;
        true
    }
}
