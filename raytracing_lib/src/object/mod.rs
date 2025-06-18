use nalgebra_glm::Vec3;

mod sphere;

pub trait Position {
    fn position(&self) -> &Vec3;
}

pub use sphere::Sphere;
