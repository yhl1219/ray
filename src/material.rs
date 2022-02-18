use crate::config::*;
use crate::common::HitRecord;

use rand::prelude::*;

// #[derive(Debug, Clone, Copy)]
// pub struct BRDF {
//     // bling-phong lighting model
//     specular: Fp,
//     diffusion: Fp,
//     refraction: Fp,

//     rho_d: Fp,
//     rho_s: Fp,
//     phong_s: Fp,
//     refration_rate: Fp,
// }

// pub enum PresetBRDF {
//     Diffuse,
//     Mirror,
//     Glass,
//     Light,
//     Marble,
//     Floor,
//     Wall,
//     Desk,
//     StanfordModel,
//     Water,
//     Teapot,
//     Metal,
// }

// impl BRDF {
//     pub fn new(spec: Fp, diff: Fp, refr: Fp, rhod: Fp, rhos: Fp, phongs: Fp, refn: Fp) -> BRDF {
//         BRDF {
//             specular: spec,
//             diffusion: diff,
//             refraction: refr,
//             rho_d: rhod,
//             rho_s: rhos,
//             phong_s: phongs,
//             refration_rate: refn,
//         }
//     }

//     pub fn load_preset(preset: PresetBRDF) -> BRDF {
//         match preset {
//             PresetBRDF::Diffuse => BRDF::new(0.0, 1.0, 0.0, 0.7, 0.0, 0.0, 0.0),
//             PresetBRDF::Mirror => BRDF::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
//             PresetBRDF::Glass => BRDF::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.65),
//             PresetBRDF::Light => BRDF::new(0.0, 1.0, 0.0, 0.7, 0.0, 0.0, 0.0),
//             PresetBRDF::Marble => todo!(),
//             PresetBRDF::Floor => todo!(),
//             PresetBRDF::Wall => todo!(),
//             PresetBRDF::Desk => todo!(),
//             PresetBRDF::StanfordModel => todo!(),
//             PresetBRDF::Water => todo!(),
//             PresetBRDF::Teapot => todo!(),
//             PresetBRDF::Metal => todo!(),
//         }
//     }
// }

pub trait Material {
    // ds: scatter direction (unit vector), di: incident direction (unit vector)
    fn scatter(&self, di: &Vector3f, rec: &HitRecord) -> Option<(Vector3f, Color3f)>;
    // passive light emission.
    fn emit(&self, _di: &Vector3f, _rec: &HitRecord) -> Color3f { Color3f::zeros() }

    fn is_diffuse(&self) -> bool { false }
    fn bsdf(&self, _wo: &Vector3f, _wi: &Vector3f) -> Color3f { Color3f::zeros() }
    // fn query(&self, position: &Vector3f) -> Vector3f;
}

// #[derive(Debug, Clone, Copy)]
// pub struct PureMaterial {
//     brdf: BRDF,
//     color: Vector3f,
// }

// impl PureMaterial {
//     pub fn new(brdf: BRDF, color: Vector3f) -> PureMaterial {
//         PureMaterial {
//             brdf: brdf,
//             color: color,
//         }
//     }
// }

// impl Material for PureMaterial {
//     fn query(&self, _position: &Vector3f) -> Vector3f {
//         self.color
//     }
// }

pub struct Lambertian {
    albedo: Color3f,
}

impl Lambertian {
    pub fn new(albedo: Color3f) -> Self {
        Lambertian { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _di: &Vector3f, rec: &HitRecord) -> Option<(Vector3f, Color3f)> {
        // build scattering coordinates (uvw basis)
        let w = rec.norm;
        let a = if w.x.abs() > 0.9 { Vector3f::y() } else { Vector3f::z() };
        let u = w.cross(&a).normalize();
        let v = w.cross(&u);

        // random uniform direction on hemisphere
        let r1: Fp = random();
        let r2: Fp = random();
        let phi = 2. * consts::PI * r1;
        let r2s = r2.sqrt(); // sin_theta

        let x = phi.cos() * r2s;
        let y = phi.sin() * r2s;
        let z = (1. - r2).sqrt(); // cos_theta
        let ds = x * u + y * v + z * w;

        Some((ds, self.albedo))
    }

    fn is_diffuse(&self) -> bool { true }
    fn bsdf(&self, _wo: &Vector3f, _wi: &Vector3f) -> Color3f {
        self.albedo * consts::FRAC_1_PI
    }
}

// v: input ray direction; n: normal
fn reflect(v: Vector3f, n: Vector3f) -> Vector3f {
    v - 2. * v.dot(&n) * n
}

pub struct Mirror {
    albedo: Color3f,
}

impl Mirror {
    pub fn new(albedo: Color3f) -> Self {
        Mirror { albedo }
    }
}

impl Material for Mirror {
    fn scatter(&self, di: &Vector3f, rec: &HitRecord) -> Option<(Vector3f, Color3f)> {
        let ds = reflect(*di, rec.norm);
        Some((ds, self.albedo))
    }
}

pub struct Dialectric {
    ir: Fp, // index of refraction
}

impl Dialectric {
    pub fn new(index_of_refraction: Fp) -> Self {
        Dialectric { ir: index_of_refraction }
    }
}

impl Material for Dialectric {
    fn scatter(&self, di: &Vector3f, rec: &HitRecord) -> Option<(Vector3f, Color3f)> {
        let outward_normal = rec.norm;
        let from_inside = di.dot(&outward_normal) > 0.0;
        let n = if from_inside { -outward_normal } else { outward_normal }; // face normal

        // eta = n_i / n_t
        // n_i: index of refraction from incident side; n_t: index of refraction from transmitted side
        let eta = if from_inside { self.ir } else { 1. / self.ir };
        let ddn = di.dot(&n); // d dot n, = - cos(i)
        let sin2t = eta * eta * (1. - ddn * ddn); // sin(t) squared, not sin(2t)
        
        let scatter_dir: Vector3f;
        if sin2t > 1. { // total internal reflection
            scatter_dir = reflect(*di, n);
        } else {
            // calculate reflectance using Schlick's approximation
            let r0 = ((eta - 1.) / (eta + 1.)).powi(2);
            let re = r0 + (1. - r0) * (1. + ddn).powi(5);

            // choose reflection or refraction
            if random::<Fp>() < re {
                scatter_dir = reflect(*di, n);
            } else {
                let cost = (1. - sin2t).sqrt();
                scatter_dir = -cost * n + eta * (*di - ddn * n); // refracted direction
            }
        }
        
        Some((scatter_dir, Color3f::new(1.0, 1.0, 1.0)))
    }
}

pub struct DiffuseLight {
    c: Color3f
}

impl DiffuseLight {
    pub fn new(c: Color3f) -> Self {
        DiffuseLight { c }
    }
}

impl Material for DiffuseLight {
    fn scatter(&self, _di: &Vector3f, _rec: &HitRecord) -> Option<(Vector3f, Color3f)> {
        None
    }

    fn emit(&self, _di: &Vector3f, _rec: &HitRecord) -> Color3f {
        self.c
    }

    fn is_diffuse(&self) -> bool {
        true
    }
}
