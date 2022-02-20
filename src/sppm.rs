use crate::config::*;
use crate::common::*;
use crate::render::*;

use std::sync::{Arc, Mutex};
use std::ops::DerefMut;
use image::{ImageBuffer, Rgb, RgbImage};
use rayon::prelude::*;
use rand::prelude::*;

// SPPM configuration
const ALPHA: Fp = 0.7;
const INITIAL_RADIUS: Fp = 1.0;
const HASH_SIZE: usize = (IMG_HEIGHT * IMG_WIDTH) as usize;
const N_PIXELS: usize = (IMG_HEIGHT * IMG_WIDTH) as usize;
const PHOTONS_PER_ITER: usize = 200000;

#[derive(Clone)]
struct VisiblePoint {
    pos: Point3f,
    beta: Color3f,
    f: Color3f,
}

#[derive(Clone)]
struct SPPMPixel {
    vp: VisiblePoint,
    ld: Color3f,
    r2: Fp,
    n: Fp,
    m: i32,
    phi: Color3f,
    tau: Color3f,
}

struct SPPMRenderer {
    pixels: Vec<Arc<Mutex<SPPMPixel>>>,
    grids: Vec<Vec<Arc<Mutex<SPPMPixel>>>>,
    grid_bound: AABB,
    inv_cell_size: Fp,
}

impl VisiblePoint {
    pub fn new() -> Self {
        Self {
            pos: Point3f::from(Vector3f::zeros()),
            beta: Color3f::zeros(),
            f: Color3f::zeros(),
        }
    }
}

impl SPPMPixel {
    pub fn new() -> Self {
        Self {
            vp: VisiblePoint::new(),
            ld: Color3f::zeros(),
            r2: INITIAL_RADIUS * INITIAL_RADIUS,
            n: 0.0,
            m: 0,
            phi: Color3f::zeros(),
            tau: Color3f::zeros(),
        }
    }
}

fn hash(i: i32, j: i32, k: i32) -> usize {
    let a = (i * 73856093) as usize;
    let b = (j * 19349663) as usize;
    let c = (k * 83492791) as usize;
    (a ^ b ^ c) % HASH_SIZE
}

impl SPPMRenderer {
    pub fn new() -> Self {
        Self {
            pixels: vec![],
            grids: vec![],
            grid_bound: Default::default(),
            inv_cell_size: 0.0
        }
    }

    fn trace(&self, scene: &Scene, ray: &Ray, beta: Color3f, pixel: &mut Option<&mut SPPMPixel>, depth: i32) {
        if depth >= MAX_DEPTH {
            return;
        }

        let mut rec = HitRecord::new();
        if !scene.intersect(&mut rec, ray) {
            // TODO: for camera pass, add direct illumination to pixel
            return;
        }

        if let Some(mat) = &rec.mat {
            if let Some(pixel) = pixel {
                // camera pass
                pixel.ld += mat.emit(&ray.d, &rec);

                if mat.is_diffuse() {
                    let vp = &mut pixel.vp;
                    vp.pos = rec.pos;
                    vp.beta = beta;
                    // TODO: use correct BSDF instead
                    // NOTE: diffuse BSDF should be a constant, so the arguments only serve as placeholders
                    vp.f = mat.bsdf(&ray.d, &ray.d);
                    return;
                }
            } else {
                // photon pass
                let (i, j, k) = self.to_grid(&rec.pos);
                let h = hash(i, j, k);
                for mutex in &self.grids[h] {
                    let mut guard = mutex.lock().unwrap();
                    let pixel = guard.deref_mut();
                    let dist_squared = (pixel.vp.pos - rec.pos).norm_squared();
                    if dist_squared > pixel.r2 {
                        continue;
                    }

                    let f = pixel.vp.f;
                    pixel.phi += beta.component_mul(&f);
                    pixel.m += 1;
                }
            }

            // TODO: Russian Roulette

            let scatter_result = mat.scatter(&ray.d, &rec);
            if let Some((scatter_dir, attenuation)) = scatter_result {
                let scattered_ray = Ray::new(rec.pos, scatter_dir);
                self.trace(scene, &scattered_ray, beta.component_mul(&attenuation), pixel, depth + 1);
            }
        }
    }

    fn to_grid(&self, p: &Point3f) -> (i32, i32, i32) {
        let g = (p - self.grid_bound.min) * self.inv_cell_size;
        (g.x as i32, g.y as i32, g.z as i32)
    }

    fn build_hash_grid(&mut self) {
        self.grids.clear();
        self.grids.resize(HASH_SIZE, vec![]);

        // compute grid bounds
        let mut max_radius: Fp = 0.0;
        let mut grid_bound = Default::default();
        for pixel in &self.pixels {
            let px = pixel.lock().unwrap();
            let r = px.r2.sqrt();
            let p = px.vp.pos;
            let vp_bound = AABB::new(
                p - Vector3f::new(r, r, r),
                p + Vector3f::new(r, r, r)
            );
            grid_bound = AABB::union(&grid_bound, &vp_bound);
            max_radius = max_radius.max(r);
        }
        self.grid_bound = grid_bound;

        // let cell size equals to max radius (or double?)
        self.inv_cell_size = 1.0 / max_radius;

        for pixel in &self.pixels {
            let px = pixel.lock().unwrap();
            if near_zero(&px.vp.beta) {
                continue;
            }

            let r = px.r2.sqrt();
            let p = px.vp.pos;
            let rv = Vector3f::new(r, r, r);
            let (i_min, j_min, k_min) = self.to_grid(&(p - rv));
            let (i_max, j_max, k_max) = self.to_grid(&(p + rv));
            for i in i_min..=i_max {
                for j in j_min..=j_max {
                    for k in k_min..=k_max {
                        let h = hash(i, j, k);
                        self.grids[h].push(pixel.clone());
                    }
                }
            }
        }
    }

    pub fn render(&mut self, scene: &Scene) {
        self.pixels.clear();
        for _ in 0..N_PIXELS {
            self.pixels.push(Arc::new(Mutex::new(SPPMPixel::new())));
        }
        
        let nr_rounds = 100;
        for round in 1..=nr_rounds {
            // camera pass
            for y in 0..IMG_HEIGHT {
                (0..IMG_WIDTH).into_par_iter().for_each(|x| {
                    let mut rng = thread_rng();
                    let ray = scene.camera.emit(x, y, &mut rng);

                    let pixel_index = (y * IMG_WIDTH + x) as usize;
                    let mut guard = self.pixels[pixel_index].lock().unwrap();
                    let pixel = guard.deref_mut();
                    self.trace(scene, &ray, Color3f::new(1., 1., 1.),
                        &mut Some(pixel), 0);
                });
            }
            println!("[round {}] finish camera pass", round);

            self.build_hash_grid();
            println!("[round {}] finish building hash grid", round);
            
            // photon pass
            let nr_lights = scene.lights.len();
            (0..PHOTONS_PER_ITER).into_par_iter().for_each(|i| {
                let mut rng = thread_rng();
                let light = &scene.lights[i % nr_lights];
                let (ray, f) = light.emit(&mut rng);
                self.trace(scene, &ray, f, &mut None, 0);
            });
            println!("[round {}] finish photon pass", round);

            // update radius
            self.pixels.par_iter().for_each(|mutex| {
                let mut px = mutex.lock().unwrap();
                if px.m > 0 {
                    let m = px.m as Fp;
                    let n_new = px.n + ALPHA * m;
                    let g = n_new / (px.n + m);
                    px.tau = (px.tau + px.vp.beta.component_mul(&px.phi)) * g;
                    px.r2 *= g;
                    px.n = n_new;

                    px.phi = Color3f::zeros();
                    px.m = 0;
                }
                px.vp.beta = Color3f::zeros();
            });

            // save image
            if round % 5 != 0 {
                continue;
            }

            let gamma = |x: Fp| x.powf(1.0 / 2.2);
            let nr_photons = PHOTONS_PER_ITER * round;
            let mut buf: RgbImage = ImageBuffer::new(IMG_WIDTH, IMG_HEIGHT);
            for (i, mutex) in self.pixels.iter().enumerate() {
                let px = mutex.lock().unwrap();
                let mut c = px.ld / round as Fp;
                // let mut c = Color3f::zeros();
                c += px.tau / (nr_photons as Fp * consts::PI * px.r2);
                c = 256. * c.apply_into(|x| *x = gamma(*x));
                
                let r = c.x.clamp(0., 255.) as u8;
                let g = c.y.clamp(0., 255.) as u8;
                let b = c.z.clamp(0., 255.) as u8;
                let x = i as u32 % IMG_WIDTH;
                let y = i as u32 / IMG_WIDTH;
                buf.put_pixel(x, IMG_HEIGHT - 1 - y, Rgb([r, g, b]));
            }
            let path = format!("round-{}.jpg", round);
            buf.save(path).unwrap();
        }
    }
}

pub fn render_sppm(scene: &Scene, _output_path: &str) {
    let mut renderer = SPPMRenderer::new();
    renderer.render(scene);
}
