use na::Vector3;

extern crate nalgebra as na;

// default precision used for calculation
pub type Fp = f64;

pub type Vector3f = Vector3<Fp>;
pub type Color3f = Vector3<Fp>;
pub type Point3f = na::Point3<Fp>; 
pub type Affine3f = na::Affine3<Fp>;

pub const IMG_WIDTH: u32 = 600;
pub const IMG_HEIGHT: u32 = 600;

pub const SAMPLES_PER_PIXEL: u32 = 500;

pub const T_MIN: Fp = 1e-4;
pub const MAX_DEPTH: i32 = 10;

pub use std::f64::consts;
