use nalgebra_glm::Vec3;

use crate::Ray;

#[derive(Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    #[inline]
    pub fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> bool {
        for a in 0..3 {
            let inv_d: f32 = 1f32 / r.direction[a];
            let mut t0: f32 = (self.min[a] - r.origin[a]) * inv_d;
            let mut t1: f32 = (self.max[a] - r.origin[a]) * inv_d;
            if inv_d < 0f32 {
                std::mem::swap(&mut t0, &mut t1);
            }
            let t_min = if t0 > t_min { t0 } else { t_min };
            let t_max = if t1 < t_max { t1 } else { t_max };
            if t_max <= t_min {
                return false;
            }
        }
        true
    }

    pub fn surrounding_box(box0: &Self, box1: &Self) -> Self {
        let small = Vec3::new(
            f32::min(box0.min[0], box1.min[0]),
            f32::min(box0.min[1], box1.min[1]),
            f32::min(box0.min[2], box1.min[2]),
        );
        let big = Vec3::new(
            f32::max(box0.max[0], box1.max[0]),
            f32::max(box0.max[1], box1.max[1]),
            f32::max(box0.max[2], box1.max[2]),
        );
        AABB {
            min: small,
            max: big,
        }
    }
}
