use crate::ray::Ray;
use nalgebra_glm::{vec3, Vec3};
pub struct Camera {
    origin: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    lower_left_corner: Vec3,
}

impl Camera {
    pub fn new(origin: Vec3, vertical_fov: f32, aspect_ratio: f32) -> Self {
        let h: f32 = f32::tan(vertical_fov * std::f32::consts::PI / 360.0f32);
        let horizontal = vec3(aspect_ratio * 2.0 * h, 0.0, 0.0);
        let vertical = vec3(0.0, 2.0 * h, 0.0);
        Camera {
            origin,
            horizontal,
            vertical,
            lower_left_corner: origin - horizontal / 2f32 - vertical / 2f32 - vec3(0f32, 0f32, 1.0),
        }
    }
    pub fn get_ray_from_coords(&self, u: f32, v: f32) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left_corner + u * self.horizontal + v * self.vertical - self.origin,
        )
    }
}
