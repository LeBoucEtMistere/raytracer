use super::Material;
use crate::collision::HitRecord;
use crate::ray::Ray;
use crate::utils::random_in_unit_sphere;
use nalgebra_glm::{dot, normalize, reflect_vec, Vec3};

pub struct Metal {
    pub albedo: Vec3,
    pub fuziness: f32,
}

impl Metal {
    pub fn new(albedo: Vec3, fuziness: f32) -> Self {
        let fuziness = f32::min(1.0, f32::max(0.0, fuziness));
        Metal { albedo, fuziness }
    }
}

impl Material for Metal {
    // returns None if no ray is scattered
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<Ray> {
        let reflected = normalize(&reflect_vec(&ray_in.direction, &hit_record.normal));
        let reflected = reflected + self.fuziness * random_in_unit_sphere();
        if dot(&reflected, &hit_record.normal) > 0.0 {
            Some(Ray::new(hit_record.point, reflected))
        } else {
            None
        }
    }
    // returns the albedo or attenuation of the surface
    fn albedo(&self) -> &Vec3 {
        &self.albedo
    }
}

impl Default for Metal {
    fn default() -> Self {
        Metal {
            albedo: Vec3::new(0.5, 0.5, 0.5),
            fuziness: 0.0,
        }
    }
}
