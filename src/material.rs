use crate::config::{Fp, Vector3f};

#[derive(Debug, Clone, Copy)]
pub struct BRDF {
    // bling-phong lighting model
    specular: Fp,
    diffusion: Fp,
    refraction: Fp,

    rho_d: Fp,
    rho_s: Fp,
    phong_s: Fp,
    refration_rate: Fp,
}

pub enum PresetBRDF {
    Diffuse,
    Mirror,
    Glass,
    Light,
    Marble,
    Floor,
    Wall,
    Desk,
    StanfordModel,
    Water,
    Teapot,
    Metal,
}

impl BRDF {
    pub fn new(spec: Fp, diff: Fp, refr: Fp, rhod: Fp, rhos: Fp, phongs: Fp, refn: Fp) -> BRDF {
        BRDF {
            specular: spec,
            diffusion: diff,
            refraction: refr,
            rho_d: rhod,
            rho_s: rhos,
            phong_s: phongs,
            refration_rate: refn,
        }
    }

    pub fn load_preset(preset: PresetBRDF) -> BRDF {
        match preset {
            PresetBRDF::Diffuse => BRDF::new(0.0, 1.0, 0.0, 0.7, 0.0, 0.0, 0.0),
            PresetBRDF::Mirror => BRDF::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            PresetBRDF::Glass => BRDF::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.65),
            PresetBRDF::Light => BRDF::new(0.0, 1.0, 0.0, 0.7, 0.0, 0.0, 0.0),
            PresetBRDF::Marble => todo!(),
            PresetBRDF::Floor => todo!(),
            PresetBRDF::Wall => todo!(),
            PresetBRDF::Desk => todo!(),
            PresetBRDF::StanfordModel => todo!(),
            PresetBRDF::Water => todo!(),
            PresetBRDF::Teapot => todo!(),
            PresetBRDF::Metal => todo!(),
        }
    }
}

trait Material {
    fn query(&self, position: &Vector3f) -> Vector3f;
}

#[derive(Debug, Clone, Copy)]
pub struct PureMaterial {
    brdf: BRDF,
    color: Vector3f,
}

impl PureMaterial {
    pub fn new(brdf: BRDF, color: Vector3f) -> PureMaterial {
        PureMaterial {
            brdf: brdf,
            color: color,
        }
    }
}

impl Material for PureMaterial {
    fn query(&self, _position: &Vector3f) -> Vector3f {
        self.color
    }
}
