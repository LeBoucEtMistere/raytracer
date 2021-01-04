use super::Position;
use crate::{
    aabb::AABB,
    collision::{HitRecord, Hittable},
};
use crate::{material::Material, ray::Ray};
use nalgebra_glm::{dot, length2, Vec3};
use std::sync::Arc;

pub struct Sphere {
    center: Vec3,
    radius: f32,

    material: Arc<Box<dyn Material>>,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Arc<Box<dyn Material>>) -> Self {
        Sphere {
            center,
            radius,
            material,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = r.origin - self.center;
        let a = length2(&r.direction);
        let half_b = dot(&oc, &r.direction);
        let c = length2(&oc) - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant > 0f32 {
            let root = f32::sqrt(discriminant);

            let mut temp = (-half_b - root) / a;
            if temp < t_max && temp > t_min {
                let point = r.at(temp);
                let outward_normal = (point - self.center) / self.radius;
                return Some(HitRecord::new(
                    r,
                    temp,
                    &outward_normal,
                    Arc::clone(&self.material),
                ));
            }

            temp = (-half_b + root) / a;
            if temp < t_max && temp > t_min {
                let point = r.at(temp);
                let outward_normal = (point - self.center) / self.radius;
                return Some(HitRecord::new(
                    r,
                    temp,
                    &outward_normal,
                    Arc::clone(&self.material),
                ));
            }
        }
        None
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<crate::aabb::AABB> {
        Some(AABB {
            min: self.center - Vec3::new(self.radius, self.radius, self.radius),
            max: self.center + Vec3::new(self.radius, self.radius, self.radius),
        })
    }
}

impl Position for Sphere {
    fn position(&self) -> &Vec3 {
        &self.center
    }
}
