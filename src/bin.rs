use raytracing_lib::export::PPMWriter;
use raytracing_lib::material::*;
use raytracing_lib::object::*;
use raytracing_lib::*;

use crossbeam_channel::unbounded;
use indicatif::{ProgressBar, ProgressStyle};
use nalgebra_glm::Vec3;
use rand::prelude::*;
use std::{error::Error, path::PathBuf, sync::Arc};
use threadpool::ThreadPool;

fn ray_color(world: &Arc<HittableList>, r: &Ray, depth: usize) -> Vec3 {
    if depth <= 0 {
        return Vec3::new(0.0, 0.0, 0.0);
    }

    if let Some(record) = world.hit(r, 0.001f32, f32::INFINITY) {
        if let Some(scattered) = record.material_hit.scatter(r, &record) {
            // a ray is scattered by the material
            let albedo = record.material_hit.albedo();

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
    let aspect_ratio = 3.0f32 / 2.0f32;
    let image_width = 1200usize;
    let image_height = (image_width as f32 / aspect_ratio) as usize;
    let mut cv = Canvas::new_initialized(image_height, image_width);

    // Camera
    let look_from = Vec3::new(13.0, 2.0, 3.0);
    let look_at = Vec3::new(0.0, 0.0, 0.0);
    let camera = Camera::builder()
        .set_origin(look_from)
        .set_look_at(look_at)
        .set_v_up(Vec3::new(0.0, 1.0, 0.0))
        .set_focus(FocusData {
            aperture: 0.1f32,
            focus_distance: 10.0,
        })
        .set_vertical_fov(20.0)
        .build();

    // Materials
    let mut material_atlas = MaterialAtlas::new();
    material_atlas.insert_material("GroundMat", Diffuse::new(Vec3::new(0.5, 0.5, 0.5)));

    // Add ground
    let mut world_builder = World::builder();
    world_builder.add_object(Sphere::new(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        material_atlas.get_material("GroundMat").unwrap(),
    ));

    // Add multiple small random spheres
    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random::<f32>();
            let center = Vec3::new(
                a as f32 + 0.9 * random::<f32>(),
                0.2,
                b as f32 + 0.9 * random::<f32>(),
            );

            if (center - Vec3::new(4.0, 0.2, 0.0)).norm() > 0.9 {
                let mat_name = format!("mat_{}_{}", a, b);

                if choose_mat < 0.8 {
                    // diffuse
                    let albedo = Vec3::new(random(), random(), random());
                    material_atlas.insert_material(&mat_name, Diffuse::new(albedo));
                } else if choose_mat < 0.95 {
                    // metal
                    let albedo = Vec3::new(random(), random(), random());
                    let fuzz = random::<f32>() / 2.0;
                    material_atlas.insert_material(&mat_name, Metal::new(albedo, fuzz));
                } else {
                    // glass
                    material_atlas.insert_material(&mat_name, Dielectric::new(1.5));
                }

                world_builder.add_object(Sphere::new(
                    center,
                    0.2,
                    material_atlas.get_material(&mat_name).unwrap(),
                ));
            }
        }
    }

    // Create 3 large spheres

    material_atlas.insert_material("Large1", Dielectric::new(1.5));
    world_builder.add_object(Sphere::new(
        Vec3::new(0.0, 1.0, 0.0),
        1.0,
        material_atlas.get_material("Large1").unwrap(),
    ));

    material_atlas.insert_material("Large2", Diffuse::new(Vec3::new(0.4, 0.2, 0.1)));
    world_builder.add_object(Sphere::new(
        Vec3::new(-4.0, 1.0, 0.0),
        1.0,
        material_atlas.get_material("Large2").unwrap(),
    ));

    material_atlas.insert_material("Large3", Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.0));
    world_builder.add_object(Sphere::new(
        Vec3::new(4.0, 1.0, 0.0),
        1.0,
        material_atlas.get_material("Large3").unwrap(),
    ));

    let world = world_builder.build();

    // Render
    let number_samples = 300usize;
    let max_depth = 50usize;

    // Progress tracking
    let (tx, rx) = unbounded::<()>();
    let progress_thread_handle = std::thread::spawn(move || {
        println!("Starting render ...");
        let pb = ProgressBar::new(number_samples as u64);
        pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} pass(es) ({eta})")
        .progress_chars("#>-"));
        pb.set_position(0);
        let mut done = 0usize;
        for _msg in rx.iter() {
            done += 1;
            pb.set_position(done as u64);
        }
        pb.finish();
    });

    let (data_tx, data_rx) = unbounded::<Canvas>();
    let tp = ThreadPool::default();

    for _ in 0..number_samples {
        let camera_arc = Arc::clone(&camera);
        let hittables_arc = world.get_hittables();
        let tx_clone = tx.clone();
        let data_tx_clone = data_tx.clone();

        tp.execute(move || {
            let result = render(
                image_height,
                image_width,
                camera_arc,
                hittables_arc,
                max_depth,
            );
            tx_clone.send(()).unwrap();
            data_tx_clone.send(result).unwrap();
        })
    }

    tp.join();

    cv = data_rx
        .iter()
        .take(number_samples)
        .fold(cv, |acc, b| acc + b);

    cv.normalize();
    cv.gamma_correction();

    drop(tx); // close channel by dropping last alive Sender
    progress_thread_handle.join().unwrap();

    // Saving image
    let p = PathBuf::from("laifilfse.ppm");
    cv.write_to_file(&p).map_err(|e| e.into())
}
