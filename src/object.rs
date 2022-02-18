use crate::config::{Fp, Point3f, T_MIN};
use crate::common::{Ray, HitRecord};
use crate::material::Material;

use std::sync::Arc;

type DynMaterial = dyn Material + Sync + Send;

pub trait Object {
    fn intersect(&self, rec: &mut HitRecord, ray: &Ray) -> bool;
}

pub struct Sphere {
    c: Point3f,
    r: Fp,
    mat: Arc<DynMaterial>
}

impl Sphere {
    pub fn new(center: Point3f, radius: Fp, material: Arc<DynMaterial>) -> Sphere {
        Sphere {
            c: center,
            r: radius,
            mat: material,
        }
    }
}

impl Object for Sphere {
    fn intersect(&self, rec: &mut HitRecord, ray: &Ray) -> bool {
        let oc = ray.o - self.c;
        // let a = ray.d.norm_squared(); // assert_eq!(a, 1.0);
        let a = 1.0;
        let half_b = oc.dot(&ray.d);
        let c = oc.norm_squared() - self.r * self.r;

        let delta = half_b * half_b - a * c;
        if delta < 0.0 {
            return false
        }

        let sqrtd = delta.sqrt();
        let mut t = (-half_b - sqrtd) / a;
        if t < T_MIN {
            t = (-half_b + sqrtd) / a;
            if t < T_MIN {
                return false
            }
        }

        rec.t = t;
        rec.pos = ray.at(t);
        rec.norm = (rec.pos - self.c) / self.r; // outward normal
        rec.mat = Some(self.mat.clone());
        true
    }
}
