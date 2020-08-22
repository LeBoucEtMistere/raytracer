use crate::material::Material;
use crate::ray::Ray;
use nalgebra_glm::{dot, Vec3};
use std::sync::Arc;

pub struct HitRecord {
    pub point: Vec3,
    pub normal: Vec3,
    pub t: f32,
    pub front_face: bool,
    pub material_hit: Arc<Box<dyn Material>>,
}

impl HitRecord {
    pub fn new(
        r: &Ray,
        t: f32,
        outward_normal: &Vec3,
        material_hit: Arc<Box<dyn Material>>,
    ) -> Self {
        let front_face = dot(&r.direction, outward_normal) < 0f32;
        let normal = if front_face {
            outward_normal.clone()
        } else {
            -outward_normal.clone()
        };
        HitRecord {
            point: r.at(t),
            normal,
            t,
            front_face,
            material_hit,
        }
    }
}
pub trait Hittable: Send + Sync {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}

#[derive(Default)]
pub struct HittableList {
    hittables: Vec<Arc<dyn Hittable>>,
}

impl HittableList {
    pub fn new() -> Self {
        HittableList {
            hittables: Vec::new(),
        }
    }

    pub fn add_hittable(&mut self, hittable: Arc<dyn Hittable>) {
        self.hittables.push(hittable);
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.hittables.clear();
    }

    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut closest_hit_record: Option<HitRecord> = None;

        let mut closest_so_far = t_max;

        for object in &self.hittables {
            if let Some(record) = object.hit(ray, t_min, closest_so_far) {
                closest_so_far = record.t;
                closest_hit_record = Some(record);
            }
        }

        closest_hit_record
    }
}
