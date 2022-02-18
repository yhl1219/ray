use crate::config::*;
use crate::common::*;
use crate::object::{Object};
use crate::light::{Light};

use std::sync::Arc;
use rand::prelude::*;
use image::{ImageBuffer, Rgb, RgbImage};
use rayon::prelude::*;
use progressing::{Baring, mapping::Bar as MappingBar};

pub type Primitive = dyn Object + Sync + Send;
pub type DynLight = dyn Light + Sync + Send;

pub struct Camera {
    pos: Point3f,
    lower_left: Point3f,
    horizontal: Vector3f,
    vertical: Vector3f,
}

pub struct Scene {
    pub objects: Vec<Arc<Primitive>>,
    pub lights: Vec<Arc<DynLight>>,
    pub camera: Camera,
}

impl Camera {
    pub fn new(pos: Point3f, look_at: Point3f, up: Vector3f, fov: Fp) -> Self {
        let aspect_ratio = IMG_WIDTH as Fp / IMG_HEIGHT as Fp;
        let theta = fov.to_radians();
        let h = (theta * 0.5).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;
        let _focal_length: Fp = 1.0;

        let w = (pos - look_at).normalize();
        let u = up.cross(&w).normalize();
        let v = w.cross(&u);

        let horizontal = viewport_width * u;
        let vertical = viewport_height * v;
        let lower_left = pos - horizontal * 0.5 - vertical * 0.5 - w;

        Camera {
            pos,
            lower_left,
            horizontal,
            vertical
        }
    }

    pub fn emit(&self, x: u32, y: u32, rng: &mut ThreadRng) -> Ray {
        let u = (x as Fp + rng.gen::<Fp>()) / IMG_WIDTH as Fp;
        let v = (y as Fp + rng.gen::<Fp>()) / IMG_HEIGHT as Fp;
        Ray::new(self.pos, self.lower_left + u * self.horizontal + v * self.vertical - self.pos)
    }
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Scene {
            objects: vec![],
            lights: vec![],
            camera,
        }
    }

    pub fn add(&mut self, obj: Arc<Primitive>) {
        self.objects.push(obj);
    }

    pub fn add_light(&mut self, light: Arc<DynLight>) {
        self.lights.push(light);
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
                    *rec = tmp_rec.clone();
                }
            }
        }
        hit
    }
}

fn shade(scene: &Scene, ray: &Ray, depth: i32) -> Color3f {
    if depth >= MAX_DEPTH {
        return Color3f::zeros();
    }

    let mut rec = HitRecord::new();
    if scene.intersect(&mut rec, ray) {
        if let Some(mat) = &rec.mat {
            let scatter_result = mat.scatter(&ray.d, &rec);
            let emitted = mat.emit(&ray.d, &rec);
            if let Some((scatter_dir, attenuation)) = scatter_result {
                let scattered_ray = Ray::new(rec.pos, scatter_dir);
                return emitted + shade(scene, &scattered_ray, depth + 1).component_mul(&attenuation);
            } else { // no scatter
                return emitted;
            }
        } else {
            return 0.5 * (rec.norm + Color3f::new(1.0, 1.0, 1.0));
        }
    }

    // background color
    let t = 0.5 * (ray.d.y + 1.0);
    t * Color3f::new(0.5, 0.7, 1.0) + (1.0 - t) * Color3f::new(1.0, 1.0, 1.0) // lerp
}

pub fn render(scene: &Scene, output_path: &str) {
    let mut buf: RgbImage = ImageBuffer::new(IMG_WIDTH, IMG_HEIGHT);
    let mut results = Vec::new();

    let mut progress_bar = MappingBar::with_range(0, IMG_HEIGHT as i32).timed();
    let gamma = |x: Fp| x.powf(1.0 / 2.2);

    // render each line of image sequentially
    for y in 0..IMG_HEIGHT {
        let mut line_result: Vec<_> = (0..IMG_WIDTH).into_par_iter().map(|x| {
            let mut rng = thread_rng();
            let mut color = Vector3f::zeros();
            for _s in 0..SAMPLES_PER_PIXEL {
                let ray = scene.camera.emit(x, y, &mut rng);
                color += shade(&scene, &ray, 0);
            }

            let scale = 1. / SAMPLES_PER_PIXEL as Fp;
            let c = 256. * (scale * color).apply_into(|x| *x = gamma(*x));
            let r = c.x.clamp(0., 255.) as u8;
            let g = c.y.clamp(0., 255.) as u8;
            let b = c.z.clamp(0., 255.) as u8;
            // flip image
            ((x, IMG_HEIGHT - 1 - y), [r, g, b])
        }).collect();
        results.append(&mut line_result);
        
        progress_bar.add(1);
        println!("{}", progress_bar);
        // if progress_bar.has_progressed_significantly() {
        //     progress_bar.remember_significant_progress();
        //     println!("{}", progress_bar);
        // }
    }

    for ((x, y), pixel) in results {
        buf.put_pixel(x, y, Rgb(pixel));
    }
    buf.save(output_path).unwrap()
}
