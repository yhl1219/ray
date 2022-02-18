use crate::config::{Fp, Vector3f, Point3f};
use crate::material::Material;

use std::sync::Arc;
use std::default::Default;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub o: Point3f,
    pub d: Vector3f,
}

#[derive(Clone)]
pub struct HitRecord {
    pub pos: Point3f,
    pub norm: Vector3f,
    pub t: Fp,
    pub mat: Option<Arc<dyn Material>>
}

pub struct AABB {
    pub min: Point3f,
    pub max: Point3f,
}

impl Ray {
    pub fn new(ori: Point3f, dir: Vector3f) -> Ray {
        Ray {
            o: ori,
            d: dir.normalize()
        }
    }

    pub fn at(&self, t: Fp) -> Point3f {
        self.o + t * self.d
    }
}

impl HitRecord {
    pub fn new() -> Self {
        HitRecord {
            pos: Point3f::from(Vector3f::zeros()),
            norm: Vector3f::zeros(),
            t: 0.0,
            mat: None
        }
    }
}

impl AABB {
    pub fn new(min: Point3f, max: Point3f) -> Self {
        AABB { min, max }
    }

    pub fn union(b1: &AABB, b2: &AABB) -> AABB {
        let p = b1.min;
        let q = b2.min;
        let u = b1.max;
        let v = b2.max;
        AABB {
            min: Point3f::new(p.x.min(q.x), p.y.min(q.y), p.z.min(q.z)),
            max: Point3f::new(u.x.max(v.x), u.y.max(v.y), u.z.max(v.z))
        }
    }
}

impl Default for AABB {
    fn default() -> Self {
        AABB {
            min: Point3f::new(Fp::MAX, Fp::MAX, Fp::MAX),
            max: Point3f::new(Fp::MIN, Fp::MIN, Fp::MIN)
        }
    }
}

pub fn near_zero(v: &Vector3f) -> bool {
    let eps = 1e-6;
    v.x < eps && v.y < eps && v.z < eps 
}
