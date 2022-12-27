use raytracing_lib::*;

use nalgebra_glm::Vec3;
use std::fs;
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

    let scene: raytracing_lib::scene::serialization::Scene =
        serde_yaml::from_str(&fs::read_to_string("examples/scene.yaml")?).unwrap();

    let (_material_atlas, world) =
        std::convert::TryInto::<(MaterialAtlas, World)>::try_into(scene)?;

    // Image
    let aspect_ratio = 3.0f32 / 2.0f32;
    let image_width = 1080usize;
    let image_height = (image_width as f32 / aspect_ratio) as usize;

    // Render
    let p = PathBuf::from("laifilfse.ppm");
    Renderer::new(world, camera)
        .width(image_width)
        .height(image_height)
        .bounces(50)
        .samples(128)
        .render()
        .save(&p)
        .map_err(|err| err.into())
}
