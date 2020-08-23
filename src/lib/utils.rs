use nalgebra_glm::{length2, Vec3};
use rand::prelude::*;

pub fn rand_range_f32(min: f32, max: f32) -> f32 {
    min + (max - min) * random::<f32>()
}

pub fn random_in_unit_sphere() -> Vec3 {
    loop {
        let p = Vec3::new_random();
        if length2(&p) < 1f32 {
            return p;
        };
    }
}

pub fn random_unit_vector() -> Vec3 {
    let a = rand_range_f32(0f32, 2f32 * std::f32::consts::PI);
    let z = rand_range_f32(-1f32, 1f32);
    let r = f32::sqrt(1.0 - z * z);
    return Vec3::new(r * f32::cos(a), r * f32::sin(a), z);
}

pub fn schlick(cosine: f32, refraction_index: f32) -> f32 {
    let r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * f32::powi(1.0 - cosine, 5)
}
