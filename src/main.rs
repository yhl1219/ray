mod config;
mod material;

extern crate image;
extern crate itertools;

use config::{IMG_WIDTH, IMG_HEIGHT};
use image::{ImageBuffer, Rgb, RgbImage};
use rayon::prelude::*;
use std::env;

fn main() {
    let mut output_path = "output.jpg";

    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        output_path = &args[1];
    }

    let mut buf: RgbImage = ImageBuffer::new(IMG_WIDTH, IMG_HEIGHT);
    let points: Vec<_> = itertools::iproduct!(0..IMG_WIDTH, 0..IMG_HEIGHT).collect();
    let results: Vec<_> = points.par_iter().map(|&(x, y)| {
        ((x, y), [x as u8, y as u8, (x + y) as u8])
    }).collect();

    for ((x, y), pixel) in results {
        buf.put_pixel(x, y, Rgb(pixel));
    }
    buf.save(output_path).unwrap()
}
