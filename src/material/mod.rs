use crate::collision::HitRecord;
use crate::ray::Ray;
use nalgebra_glm::Vec3;

mod dielectric;
mod diffuse;
mod metal;

pub use dielectric::Dielectric;
pub use diffuse::Diffuse;
pub use metal::Metal;

pub trait Material: Send + Sync {
    // returns None if no ray is scattered
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<Ray>;
    // returns the albedo or attenuation of the surface
    fn albedo(&self) -> &Vec3;
}
