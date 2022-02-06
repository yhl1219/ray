mod config;
mod material;
mod common;
mod object;
mod render;

use config::*;
use render::*;
use object::*;
use material::*;

use image::{ImageBuffer, Rgb, RgbImage};
use rayon::prelude::*;
use rand::thread_rng;
use progressing::{Baring, mapping::Bar as MappingBar};

use std::env;
use std::sync::Arc;

fn build_scene() -> Scene {
    let pos = Point3f::new(-2., 2., 1.);
    let look_at = Point3f::new(0., 0., -1.);
    let up = Vector3f::new(0., 1., 0.);

    let camera = Camera::new(pos, look_at, up, 20.0);
    let mut scene = Scene::new(camera);

    // two lambertian spheres
    // let mat = Arc::new(Lambertian::new(Color3f::new(0.5, 0.5, 0.5)));
    // scene.add(Arc::new(Sphere::new(Vector3f::new(0.0, 0.0, -1.0), 0.5, mat.clone())));
    // scene.add(Arc::new(Sphere::new(Vector3f::new(0.0, -100.5, -1.0), 100.0, mat)));
    
    let mat1 = Arc::new(Lambertian::new(Color3f::new(0.8, 0.8, 0.0)));
    // let mat2 = Arc::new(Lambertian::new(Color3f::new(0.7, 0.3, 0.3)));
    // let mat3 = Arc::new(Mirror::new(Color3f::new(0.8, 0.8, 0.8)));
    let mat2 = Arc::new(Lambertian::new(Color3f::new(0.1, 0.2, 0.5)));
    let mat3 = Arc::new(Dialectric::new(1.5));
    let mat4 = Arc::new(Mirror::new(Color3f::new(0.8, 0.6, 0.2)));

    scene.add(Arc::new(Sphere::new(Point3f::new( 0., -100.5, -1.), 100.0, mat1)));
    scene.add(Arc::new(Sphere::new(Point3f::new( 0.,    0.0, -1.),   0.5, mat2)));
    scene.add(Arc::new(Sphere::new(Point3f::new(-1.,    0.0, -1.),   0.5, mat3)));
    scene.add(Arc::new(Sphere::new(Point3f::new( 1.,    0.0, -1.),   0.5, mat4)));
    scene
}

fn cornell_box() -> Scene {
    let pos = Point3f::new(278., 278., -800.);
    let look_at = Point3f::new(278., 278., 0.);
    let up = Vector3f::new(0., 1., 0.);

    let camera = Camera::new(pos, look_at, up, 40.0);
    let mut scene = Scene::new(camera);

    let red   = Arc::new(Lambertian::new(Color3f::new(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new(Color3f::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(Color3f::new(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::new(Color3f::new(6., 6., 6.)));
    let glass = Arc::new(Dialectric::new(1.5));
    let mirror = Arc::new(Mirror::new(Color3f::new(0.8, 0.8, 0.8)));

    scene.add(Arc::new(Sphere::new(Point3f::new( 10555., 278., 278.), 10000., green)));
    scene.add(Arc::new(Sphere::new(Point3f::new(-10000., 278., 278.), 10000., red)));
    scene.add(Arc::new(Sphere::new(Point3f::new(278., -10000., 278.), 10000., white.clone())));
    scene.add(Arc::new(Sphere::new(Point3f::new(278.,  10555., 278.), 10000., white.clone())));
    scene.add(Arc::new(Sphere::new(Point3f::new(278.,  278., 10555.), 10000., white.clone())));
    
    scene.add(Arc::new(Sphere::new(Point3f::new(278.,  555., 278.), 50., light)));
    scene.add(Arc::new(Sphere::new(Point3f::new(120., 90., 120.), 90., glass)));
    scene.add(Arc::new(Sphere::new(Point3f::new(360., 90., 360.), 90., mirror)));
    scene
}

fn main() {
    let mut output_path = "output.jpg";

    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        output_path = &args[1];
    }

    let scene = cornell_box();

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
                color += shade(&scene, &ray, 0, &mut rng);
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
