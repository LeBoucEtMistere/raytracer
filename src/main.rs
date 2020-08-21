mod camera;
mod canvas;
mod collision;
mod export;
mod material;
mod objects;
mod ray;
mod utils;

use camera::Camera;
use canvas::Canvas;
use collision::HittableList;
use crossbeam_utils::thread;
use export::PPMWriter;
use material::{Diffuse, Metal};
use nalgebra_glm::Vec3;
use objects::Sphere;
use rand::prelude::*;
use ray::Ray;
use std::{
    error::Error,
    path::PathBuf,
    sync::{Arc, Mutex},
};

fn ray_color(world: &Arc<HittableList>, r: &Ray, depth: usize) -> Vec3 {
    if depth <= 0 {
        return Vec3::new(0.0, 0.0, 0.0);
    }

    if let Some(record) = world.hit(r, 0.001f32, f32::INFINITY) {
        let material_lock_guard = record.material_hit.lock().unwrap();
        if let Some(scattered) = material_lock_guard.scatter(r, &record) {
            // a ray is scattered by the material
            let albedo = material_lock_guard.albedo();
            drop(material_lock_guard);
            return albedo.component_mul(&ray_color(world, &scattered, depth - 1));
        }
        // no scattered ray, return black
        return Vec3::new(0.0, 0.0, 0.0);
    } else {
        let unit_direction = nalgebra_glm::normalize(&r.direction);
        let t = 0.5 * (unit_direction.y + 1.0); // t is between 0.0 and 1.0
        nalgebra_glm::lerp(&Vec3::new(1.0, 1.0, 1.0), &Vec3::new(0.5, 0.7, 1.0), t)
    }
}

fn render(
    canvas_height: usize,
    canvas_width: usize,
    camera: Arc<Camera>,
    world: Arc<HittableList>,
    max_depth: usize,
) -> Canvas {
    let mut temp_cv = Canvas::new_initialized(canvas_height, canvas_width);
    for j in 0..canvas_height {
        for i in 0..canvas_width {
            let u = (i as f32 + random::<f32>()) / (canvas_width - 1) as f32;
            let v = (j as f32 + random::<f32>()) / (canvas_height - 1) as f32;
            let r = camera.get_ray_from_coords(u, v);

            // need to flip horizontally since Canvas has its y axis going down and camera going up
            temp_cv.set_pixel(i, canvas_height - 1 - j, ray_color(&world, &r, max_depth))
        }
    }
    temp_cv
}

fn main() -> Result<(), Box<dyn Error>> {
    // Image
    let aspect_ratio = 16.0f32 / 9.0f32;
    let image_width = 820usize;
    let image_height = (image_width as f32 / aspect_ratio) as usize;
    let mut cv = Canvas::new_initialized(image_height, image_width);

    // Camera
    let viewport_height = 2.0;
    let viewport_width = aspect_ratio * viewport_height;
    let focal_length = 1.0;
    let camera = Arc::new(Camera::new(
        Vec3::new(0.0, 0.0, 0.0),
        focal_length,
        viewport_width,
        viewport_height,
    ));

    let number_samples = 100usize;
    let max_depth = 50usize;

    // Objects
    let mut world = HittableList::new();
    let mat_diff = Arc::new(Mutex::new(Diffuse::new(Vec3::new(0.1, 0.6, 0.0))));
    let mat_metal = Arc::new(Mutex::new(Metal::new(Vec3::new(0.8, 0.6, 0.2), 1.0)));
    let obj1 = Arc::new(Mutex::new(Sphere::new(
        Vec3::new(-0.7, 0.0, -1.0),
        0.5,
        Some(mat_diff),
    )));
    world.add_hittable(obj1);
    world.add_hittable(Arc::new(Mutex::new(Sphere::new(
        Vec3::new(0.7, 0.0, -1.0),
        0.5,
        Some(mat_metal),
    ))));
    world.add_hittable(Arc::new(Mutex::new(Sphere::new(
        Vec3::new(0.0, -100.5, -1.0),
        100.0,
        None,
    ))));

    let world = Arc::new(world);

    match thread::scope(|s| -> Vec<Canvas> {
        let mut results = Vec::new();
        for _ in 0..number_samples {
            let camera_arc = Arc::clone(&camera);
            let world_arc = Arc::clone(&world);
            results.push(s.spawn(move |_| {
                render(image_height, image_width, camera_arc, world_arc, max_depth)
            }));
        }
        results.into_iter().map(|x| x.join().unwrap()).collect()
    }) {
        Ok(results) => {
            cv = results.into_iter().fold(cv, |acc, b| acc + b);
        }
        Err(_e) => eprintln!("Error in end of scoping"),
    }

    cv.normalize();
    cv.gamma_correction();

    // Saving image
    let p = PathBuf::from("laifilfse.ppm");
    cv.write_to_file(&p).map_err(|e| e.into())
}
