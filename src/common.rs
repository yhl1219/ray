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

#[derive(Debug, Clone, Copy)]
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
        AABB {
            min: point_min(&b1.min, &b2.min),
            max: point_max(&b1.max, &b2.max),
        }
    }

    pub fn center(&self) -> Point3f {
        point_mid(&self.min, &self.max)
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

fn point_op(f: fn(Fp, Fp) -> Fp, p: &Point3f, q: &Point3f) -> Point3f {
    Point3f::new(f(p.x, q.x), f(p.y, q.y), f(p.z, q.z))
}

pub fn point_min(p: &Point3f, q: &Point3f) -> Point3f {
    point_op(|x, y| { x.min(y) }, p, q)
}

pub fn point_max(p: &Point3f, q: &Point3f) -> Point3f {
    point_op(|x, y| { x.max(y) }, p, q)
}

fn point_mid(p: &Point3f, q: &Point3f) -> Point3f {
    point_op(|x, y| { (x + y) / 2.0 }, p, q)
}
