use crate::{object::Position, ray::Ray, utils::random_in_unit_disk};
use nalgebra_glm::{cross, normalize, Vec3};
use std::sync::Arc;

#[derive(Debug, Copy, Clone)]
pub struct FocusData {
    pub aperture: f32,
    pub focus_distance: f32,
}

pub struct CameraBuilder {
    origin: Vec3,
    v_up: Vec3,
    look_at: Vec3,
    vertical_fov: f32,
    aspect_ratio: f32,
    focus_data: Option<FocusData>,
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

    pub fn set_focus(mut self, focus_data: FocusData) -> Self {
        self.focus_data = Some(focus_data);
        self
    }

    pub fn build(self) -> Arc<Camera> {
        Arc::new(Camera::new(
            self.origin,
            self.vertical_fov,
            self.aspect_ratio,
            Some(self.look_at),
            Some(self.v_up),
            self.focus_data,
        ))
    }
}

#[derive(Clone)]
pub struct Camera {
    origin: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    lower_left_corner: Vec3,
    u: Vec3,
    v: Vec3,
    lens_radius: Option<f32>,
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
        focus_data: Option<FocusData>,
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

        let horizontal;
        let vertical;
        let lower_left_corner;

        if let Some(focus_data) = focus_data {
            horizontal = focus_data.focus_distance * viewport_width * u;
            vertical = focus_data.focus_distance * viewport_height * v;
            lower_left_corner =
                origin - horizontal / 2.0 - vertical / 2.0 - focus_data.focus_distance * w;
        } else {
            horizontal = viewport_width * u;
            vertical = viewport_height * v;
            lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - w;
        }

        Camera {
            origin,
            horizontal,
            vertical,
            lower_left_corner,
            u,
            v,
            lens_radius: focus_data.map(|x| x.aperture / 2.0f32),
        }
    }

    pub fn get_ray_from_coords(&self, c_u: f32, c_v: f32) -> Ray {
        match self.lens_radius {
            // if there is Some lens-radius, we need to compute defocus blur (or depth of field)
            Some(lens_radius) => {
                let rd = lens_radius * random_in_unit_disk();
                let offset = self.u * rd.x + self.v * rd.y;

                Ray::new(
                    self.origin + offset,
                    self.lower_left_corner + c_u * self.horizontal + c_v * self.vertical
                        - self.origin
                        - offset,
                )
            }
            // else we cast a normal ray from a single point
            None => Ray::new(
                self.origin,
                self.lower_left_corner + c_u * self.horizontal + c_v * self.vertical - self.origin,
            ),
        }
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
            focus_data: None,
        }
    }
}
