use crate::ray::Ray;
use nalgebra_glm::{vec3, Vec3};
pub struct Camera {
    origin: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    lower_left_corner: Vec3,
}

impl Camera {
    pub fn new(origin: Vec3, focal_length: f32, viewport_width: f32, viewport_height: f32) -> Self {
        let horizontal = vec3(viewport_width, 0.0, 0.0);
        let vertical = vec3(0.0, viewport_height, 0.0);
        Camera {
            origin,
            horizontal,
            vertical,
            lower_left_corner: origin
                - horizontal / 2f32
                - vertical / 2f32
                - vec3(0f32, 0f32, focal_length),
        }
    }
    pub fn get_ray_from_coords(&self, u: f32, v: f32) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left_corner + u * self.horizontal + v * self.vertical - self.origin,
        )
    }
}
