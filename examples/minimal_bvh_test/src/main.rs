use raytracing_lib::material::*;
use raytracing_lib::object::*;
use raytracing_lib::*;

use nalgebra_glm::Vec3;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut material_atlas = MaterialAtlas::default();
    let mut world_builder = World::builder();

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

    let ray = Ray::new(Vec3::new(-10.0, 1.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
    let bvh = world.get_hittables();
    dbg!(bvh.hit(&ray, 0.0, f32::INFINITY));
    dbg!(bvh.left.bounding_box(0.0, f32::INFINITY));
    dbg!(bvh.right.bounding_box(0.0, f32::INFINITY));

    Ok(())
}
