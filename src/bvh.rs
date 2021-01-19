use std::{cmp::Ordering, sync::Arc};

use crate::{aabb::AABB, collision::Hittable, utils};

pub struct BVHNode {
    pub left: Arc<dyn Hittable>,
    pub right: Arc<dyn Hittable>,
    pub aabb: AABB,
}

impl std::fmt::Debug for BVHNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "BVHNode: Left={:?} | Right={:?} | AABB={:?}",
            self.left.bounding_box(0.0, 0.0),
            self.right.bounding_box(0.0, 0.0),
            self.aabb
        ))
    }
}

impl BVHNode {
    pub fn new(src_hittables: &[Arc<dyn Hittable>]) -> Result<Self, String> {
        let random_axis = AxisIndexes::random_axis();

        let left_node;
        let right_node;

        match src_hittables.len() {
            0 => {
                return Err("Cannot build a BVH with an empty object list".into());
            }
            1 => {
                left_node = Arc::clone(&src_hittables[0]);
                right_node = Arc::clone(&src_hittables[0]);
            }
            2 => match box_compare(&src_hittables[0], &src_hittables[1], random_axis)? {
                Ordering::Less | Ordering::Equal => {
                    left_node = Arc::clone(&src_hittables[0]);
                    right_node = Arc::clone(&src_hittables[1]);
                }
                Ordering::Greater => {
                    left_node = Arc::clone(&src_hittables[1]);
                    right_node = Arc::clone(&src_hittables[0]);
                }
            },
            _ => {
                // need to perform a clone of the slice to sort it :/
                let mut sorted_vec = Vec::<Arc<dyn Hittable>>::new();
                sorted_vec.extend_from_slice(src_hittables);
                sorted_vec.sort_unstable_by(|a, b| box_compare(a, b, random_axis).unwrap());
                let mid = sorted_vec.len() as usize / 2;
                let (left_src, right_src) = sorted_vec.split_at(mid);
                left_node = Arc::new(BVHNode::new(left_src)?);
                right_node = Arc::new(BVHNode::new(right_src)?);
            }
        };

        let a_box = left_node.bounding_box(0f32, 0f32);
        let b_box = right_node.bounding_box(0f32, 0f32);

        if a_box.is_none() || b_box.is_none() {
            return Err("No bounding box in BVH constructor".into());
        }
        let a_box = a_box.unwrap();
        let b_box = b_box.unwrap();

        Ok(BVHNode {
            left: left_node,
            right: right_node,
            aabb: AABB::surrounding_box(&a_box, &b_box),
        })
    }
}

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

            if left_hit && right_hit {
                let rec_left = rec_left.unwrap();
                let rec_right = rec_right.unwrap();
                if rec_left.t <= rec_right.t {
                    Some(rec_left)
                } else {
                    Some(rec_right)
                }
            } else if left_hit {
                rec_left
            } else if right_hit {
                rec_right
            } else {
                None
            }
        } else {
            None
        }
    }

    fn bounding_box(&self, _t0: f32, _t1: f32) -> Option<AABB> {
        Some(self.aabb)
    }
}

trait AxisIndex {
    fn index(&self) -> usize;
}
#[derive(Copy, Clone)]
enum AxisIndexes {
    X,
    Y,
    Z,
}

impl AxisIndex for AxisIndexes {
    fn index(&self) -> usize {
        match self {
            AxisIndexes::X => 0,
            AxisIndexes::Y => 1,
            AxisIndexes::Z => 2,
        }
    }
}

impl AxisIndexes {
    #[inline]
    fn random_axis() -> Self {
        match utils::rand_range_f32(0.0, 3.0) as usize {
            0 => AxisIndexes::X,
            1 => AxisIndexes::Y,
            2 => AxisIndexes::Z,
            _ => unreachable!(),
        }
    }
}

#[inline]
fn box_compare<T: AxisIndex>(
    a: &Arc<dyn Hittable>,
    b: &Arc<dyn Hittable>,
    axis: T,
) -> Result<std::cmp::Ordering, String> {
    let a_box = a.bounding_box(0f32, 0f32);
    let b_box = b.bounding_box(0f32, 0f32);

    if a_box.is_none() || b_box.is_none() {
        return Err("No bounding box in BVH constructor".into());
    }
    let a_box = a_box.unwrap();
    let b_box = b_box.unwrap();
    Ok(a_box.min[axis.index()]
        .partial_cmp(&b_box.min[axis.index()])
        .expect("Trying to partial comp two floats returns None"))
}
