use super::Material;
use crate::collision::HitRecord;
use crate::ray::Ray;
use crate::utils::schlick;
use nalgebra_glm::{dot, normalize, reflect_vec, refract_vec, Vec3};

pub struct Dielectric {
    pub refractive_index: f32,
    albedo: Vec3,
}

impl Dielectric {
    pub fn new(refractive_index: f32) -> Self {
        Dielectric {
            refractive_index,
            albedo: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Material for Dielectric {
    // returns None if no ray is scattered
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<Ray> {
        let etai_over_etat: f32 = if hit_record.front_face {
            1.0 / self.refractive_index
        } else {
            self.refractive_index
        };

        let unit_direction = normalize(&ray_in.direction);
        let cos_theta = f32::min(dot(&-unit_direction, &hit_record.normal), 1.0);
        let sin_theta = f32::sqrt(1.0 - cos_theta * cos_theta);
        if etai_over_etat * sin_theta > 1.0f32 {
            let reflected = reflect_vec(&unit_direction, &hit_record.normal);
            return Some(Ray::new(hit_record.point, reflected));
        }
        let reflect_prob = schlick(cos_theta, etai_over_etat);
        if rand::random::<f32>() < reflect_prob {
            let reflected = reflect_vec(&unit_direction, &hit_record.normal);
            return Some(Ray::new(hit_record.point, reflected));
        }
        let refracted = refract_vec(&unit_direction, &hit_record.normal, etai_over_etat);
        return Some(Ray::new(hit_record.point, refracted));
    }
    // returns the albedo or attenuation of the surface
    fn albedo(&self) -> &Vec3 {
        &self.albedo
    }
}

impl Default for Dielectric {
    fn default() -> Self {
        Dielectric {
            refractive_index: 0.5,
            albedo: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}
