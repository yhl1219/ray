mod config;
mod material;
mod common;
mod object;
mod render;

extern crate image;
extern crate itertools;

use config::*;
use render::*;
use object::*;

use image::{ImageBuffer, Rgb, RgbImage};
use rayon::prelude::*;

use std::env;
use std::sync::Arc;

fn main() {
    let mut output_path = "output.jpg";

    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        output_path = &args[1];
    }

    let mut scene = Scene::new();
    scene.add(Arc::new(Sphere::new(Vector3f::new(0.0, 0.0, -1.0), 0.5)));
    scene.add(Arc::new(Sphere::new(Vector3f::new(0.0, -100.5, -1.0), 100.0)));

    let mut buf: RgbImage = ImageBuffer::new(IMG_WIDTH, IMG_HEIGHT);
    let points: Vec<_> = itertools::iproduct!(0..IMG_WIDTH, 0..IMG_HEIGHT).collect();
    let results: Vec<_> = points.par_iter().map(|&(x, y)| {
        let color = shade(&scene, (x, y));
        let r = (color.x * 255.999) as u8;
        let g = (color.y * 255.999) as u8;
        let b = (color.z * 255.999) as u8;
        ((x, y), [r, g, b])
    }).collect();

    for ((x, y), pixel) in results {
        buf.put_pixel(x, y, Rgb(pixel));
    }
    buf.save(output_path).unwrap()
}
