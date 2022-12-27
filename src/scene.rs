pub mod serialization {
    use serde::Deserialize;
    use std::{collections::HashMap, convert::TryFrom};

    use nalgebra_glm::Vec3;

    use crate::{
        material::{Dielectric, Diffuse, Metal},
        object::Sphere,
        MaterialAtlas, World,
    };

    #[derive(Deserialize)]
    pub struct Color3(f32, f32, f32);

    #[derive(Deserialize)]
    pub struct Point(f32, f32, f32);

    impl From<Color3> for Vec3 {
        fn from(other: Color3) -> Vec3 {
            Vec3::new(other.0, other.1, other.2)
        }
    }

    impl From<Point> for Vec3 {
        fn from(other: Point) -> Vec3 {
            Vec3::new(other.0, other.1, other.2)
        }
    }

    #[derive(Deserialize)]
    pub struct Object {
        pub object_id: String,
        pub geometry: Geometry,
        pub material: String,
    }

    #[derive(Deserialize)]
    pub enum Geometry {
        Sphere { center: Point, radius: f32 },
    }

    #[derive(Deserialize)]
    pub enum Material {
        Dielectric { refractive_index: f32 },
        Diffuse { albedo: Point },
        Metal { albedo: Point, fuziness: f32 },
    }

    #[derive(Deserialize)]
    pub struct Scene {
        objects: Vec<Object>,
        materials: HashMap<String, Material>,
    }

    impl TryFrom<Scene> for (MaterialAtlas, World) {
        type Error = String;
        fn try_from(scene: Scene) -> Result<Self, Self::Error> {
            let mut atlas = MaterialAtlas::default();
            for (name, material) in scene.materials.into_iter() {
                match material {
                    Material::Dielectric { refractive_index } => {
                        atlas.insert_material(&name, Dielectric::new(refractive_index))
                    }
                    Material::Diffuse { albedo } => {
                        atlas.insert_material(&name, Diffuse::new(albedo.into()))
                    }
                    Material::Metal { albedo, fuziness } => {
                        atlas.insert_material(&name, Metal::new(albedo.into(), fuziness))
                    }
                };
            }
            let mut world_builder = World::builder();
            for object in scene.objects.into_iter() {
                let material = &object.material;
                match object.geometry {
                    Geometry::Sphere { center, radius } => world_builder.add_object(Sphere::new(
                        center.into(),
                        radius,
                        atlas
                            .get_material(&object.material)
                            .ok_or_else(|| format!("Cannot find material {}", material))?,
                    )),
                };
            }

            Ok((atlas, world_builder.build()))
        }
    }
}
