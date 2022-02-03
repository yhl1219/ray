use crate::config::{Fp, Vector3f};
use crate::material::Material;

use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub o: Vector3f,
    pub d: Vector3f,
}

#[derive(Clone)]
pub struct HitRecord {
    pub pos: Vector3f,
    pub norm: Vector3f,
    pub t: Fp,
    pub mat: Option<Arc<dyn Material>>
}

impl Ray {
    pub fn new(ori: Vector3f, dir: Vector3f) -> Ray {
        Ray {
            o: ori,
            d: dir.normalize()
        }
    }

    pub fn at(&self, t: Fp) -> Vector3f {
        self.o + t * self.d
    }
}

impl HitRecord {
    pub fn new() -> Self {
        HitRecord {
            pos: Vector3f::zeros(),
            norm: Vector3f::zeros(),
            t: 0.0,
            mat: None
        }
    }
}
