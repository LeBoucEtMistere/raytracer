use raytracing_lib::material::*;
use raytracing_lib::object::*;
use raytracing_lib::*;

use nalgebra_glm::Vec3;
use rand::prelude::*;
use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
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

    // Image
    let aspect_ratio = 3.0f32 / 2.0f32;
    let image_width = 1200usize;
    let image_height = (image_width as f32 / aspect_ratio) as usize;

    // Render
    let p = PathBuf::from("laifilfse.ppm");
    Renderer::new(world, camera)
        .width(image_width)
        .height(image_height)
        .bounces(8)
        .samples(32)
        .render()
        .save(&p)
        .map_err(|er| er.into())
}
