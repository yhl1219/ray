use crate::config::*;
use crate::common::*;
use crate::object::{Object};

use std::sync::Arc;

pub struct Camera {
    pos: Vector3f,
    lower_left: Vector3f,
    horizontal: Vector3f,
    vertical: Vector3f,
}

pub struct Scene {
    pub objects: Vec<Arc<dyn Object + Sync + Send>>,
    pub camera: Camera,
}

impl Camera {
    pub fn new() -> Self {
        let aspect_ratio = IMG_WIDTH as Fp / IMG_HEIGHT as Fp;
        let viewport_height: Fp = 2.0;
        let viewport_width = aspect_ratio * viewport_height;
        let focal_length: Fp = 1.0;

        let pos = Vector3f::zeros();
        let horizontal = Vector3f::new(viewport_width, 0.0, 0.0);
        let vertical = Vector3f::new(0.0, viewport_height, 0.0);
        let lower_left = pos - horizontal * 0.5 - vertical * 0.5 - Vector3f::new(0.0, 0.0, focal_length);

        Camera {
            pos,
            lower_left,
            horizontal,
            vertical
        }
    }

    pub fn emit(&self, x: u32, y: u32) -> Ray {
        let u = x as Fp / (IMG_WIDTH - 1) as Fp;
        let v = y as Fp / (IMG_HEIGHT - 1) as Fp;
        Ray::new(self.pos, self.lower_left + u * self.horizontal + v * self.vertical)
    }
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            objects: vec![],
            camera: Camera::new()
        }
    }

    pub fn add(&mut self, obj: Arc<dyn Object + Sync + Send>) {
        self.objects.push(obj);
    }
}

impl Object for Scene {
    fn intersect(&self, rec: &mut HitRecord, ray: &Ray) -> bool {
        let mut hit = false;
        let mut t_min = Fp::MAX;
        for obj in &self.objects {
            let mut tmp_rec = HitRecord::new();
            if obj.intersect(&mut tmp_rec, ray) {
                hit = true;
                let t = tmp_rec.t;
                if t_min > t {
                    t_min = t;
                    // copy
                    rec.t = t;
                    rec.pos = tmp_rec.pos;
                    rec.norm = tmp_rec.norm;
                    rec.mat = tmp_rec.mat;
                }
            }
        }
        hit
    }
}

pub fn shade(scene: &Scene, (x, y): (u32, u32)) -> Color3f {
    let ray = scene.camera.emit(x, y);
    let mut rec = HitRecord::new();

    if scene.intersect(&mut rec, &ray) {
        return 0.5 * (rec.norm + Color3f::new(1.0, 1.0, 1.0));
    }

    let t = 0.5 * (ray.d.y + 1.0);
    t * Color3f::new(0.5, 0.7, 1.0) + (1.0 - t) * Color3f::new(1.0, 1.0, 1.0)
}
