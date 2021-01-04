use std::sync::Arc;

use crate::{aabb::AABB, collision::Hittable};

struct BVHNode {
    pub left: Arc<Box<dyn Hittable>>,
    pub right: Arc<Box<dyn Hittable>>,
    pub aabb: AABB,
}

impl BVHNode {}

impl Hittable for BVHNode {
    fn hit(&self, r: &crate::Ray, t_min: f32, t_max: f32) -> Option<crate::collision::HitRecord> {
        if self.aabb.hit(r, t_min, t_max) {
            let mut left_hit = false;
            let mut t = None;
            let rec_left = self.left.hit(r, t_min, t_max).map(|record| {
                t = Some(record.t);
                left_hit = true;
                record
            });

            let rec_right = self
                .right
                .hit(r, t_min, if left_hit { t.unwrap() } else { t_max });
            let right_hit = rec_right.is_some();

            if left_hit {
                return rec_left;
            } else if right_hit {
                return rec_right;
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        todo!()
    }
}
