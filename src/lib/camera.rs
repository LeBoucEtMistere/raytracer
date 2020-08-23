use crate::{object::Position, ray::Ray};
use nalgebra_glm::{cross, normalize, Vec3};
use std::sync::Arc;

pub struct CameraBuilder {
    origin: Vec3,
    v_up: Vec3,
    look_at: Vec3,
    vertical_fov: f32,
    aspect_ratio: f32,
}

impl CameraBuilder {
    pub fn set_origin(mut self, origin: Vec3) -> Self {
        self.origin = origin;
        self
    }

    pub fn set_v_up(mut self, v_up: Vec3) -> Self {
        self.v_up = v_up;
        self
    }

    pub fn set_look_at(mut self, look_at: Vec3) -> Self {
        self.look_at = look_at;
        self
    }

    pub fn set_vertical_fov(mut self, vertical_fov: f32) -> Self {
        self.vertical_fov = vertical_fov;
        self
    }

    pub fn set_aspect_ratio(mut self, aspect_ratio: f32) -> Self {
        self.aspect_ratio = aspect_ratio;
        self
    }

    pub fn build(self) -> Arc<Camera> {
        Arc::new(Camera::new(
            self.origin,
            self.vertical_fov,
            self.aspect_ratio,
            Some(self.look_at),
            Some(self.v_up),
        ))
    }
}
pub struct Camera {
    origin: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    lower_left_corner: Vec3,
}

impl Camera {
    pub fn builder() -> CameraBuilder {
        CameraBuilder::default()
    }

    fn new(
        origin: Vec3,
        vertical_fov: f32,
        aspect_ratio: f32,
        look_at: Option<Vec3>,
        v_up: Option<Vec3>,
    ) -> Self {
        let h: f32 = f32::tan(vertical_fov * std::f32::consts::PI / 360.0f32);

        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;

        let v_up = match v_up {
            Some(v) => v,
            None => Vec3::new(0.0, 1.0, 0.0),
        };

        let look_at = match look_at {
            Some(obj) => obj,
            None => Vec3::new(0.0, 0.0, -1.0),
        };

        let w = normalize(&(origin - look_at));
        let u = normalize(&cross(&v_up, &w));
        let v = cross(&w, &u);

        let horizontal = viewport_width * u;
        let vertical = viewport_height * v;
        let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - w;

        Camera {
            origin,
            horizontal,
            vertical,
            lower_left_corner,
        }
    }

    pub fn get_ray_from_coords(&self, u: f32, v: f32) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left_corner + u * self.horizontal + v * self.vertical - self.origin,
        )
    }
}

impl Position for Camera {
    fn position(&self) -> &Vec3 {
        &self.origin
    }
}

impl Default for CameraBuilder {
    fn default() -> Self {
        CameraBuilder {
            origin: Vec3::new(0.0, 0.0, 1.0),
            v_up: Vec3::new(0.0, 1.0, 0.0),
            look_at: Vec3::new(0.0, 0.0, 1.0),
            vertical_fov: 40.0,
            aspect_ratio: 16.0 / 9.0,
        }
    }
}
