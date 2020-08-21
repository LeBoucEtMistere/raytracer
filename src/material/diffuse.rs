use super::Material;
use crate::collision::HitRecord;
use crate::ray::Ray;
use crate::utils::random_unit_vector;
use nalgebra_glm::Vec3;

pub struct Diffuse {
    pub albedo: Vec3,
}

impl Diffuse {
    pub fn new(albedo: Vec3) -> Self {
        Diffuse { albedo }
    }
}

impl Material for Diffuse {
    // returns None if no ray is scattered
    #[allow(unused_variables)]
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<Ray> {
        Some(Ray::new(
            hit_record.point,
            hit_record.normal + random_unit_vector(),
        ))
    }
    // returns the albedo or attenuation of the surface
    fn albedo(&self) -> Vec3 {
        self.albedo
    }
}

impl Default for Diffuse {
    fn default() -> Self {
        Diffuse {
            albedo: Vec3::new(0.5, 0.5, 0.5),
        }
    }
}
