use crate::collision::{HitRecord, Hittable};
use crate::{material::Diffuse, material::Material, ray::Ray};
use nalgebra_glm::{dot, length2, Vec3};
use std::sync::{Arc, Mutex};

pub struct Sphere {
    center: Vec3,
    radius: f32,

    material: Arc<Mutex<dyn Material>>,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Option<Arc<Mutex<dyn Material>>>) -> Self {
        Sphere {
            center,
            radius,
            material: match material {
                Some(mat) => mat,
                None => Arc::new(Mutex::new(Diffuse::default())),
            },
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
}
