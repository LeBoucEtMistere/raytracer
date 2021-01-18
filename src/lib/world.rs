use super::bvh::BVHNode;
use super::collision::Hittable;
use std::sync::Arc;
pub struct World {
    bvh_tree: Arc<BVHNode>,
}

pub struct WorldBuilder {
    hittables: Vec<Arc<dyn Hittable>>,
}

impl WorldBuilder {
    pub fn add_object(&mut self, object: impl Hittable + 'static) -> &mut Self {
        let hittable: Arc<dyn Hittable> = Arc::new(object) as Arc<dyn Hittable>;
        self.hittables.push(hittable);
        self
    }

    pub fn build(self) -> World {
        World {
            bvh_tree: Arc::new(BVHNode::new(&self.hittables[..]).unwrap()),
        }
    }
}

impl World {
    pub fn builder() -> WorldBuilder {
        WorldBuilder {
            hittables: Vec::new(),
        }
    }

    pub fn get_hittables(&self) -> Arc<BVHNode> {
        Arc::clone(&self.bvh_tree)
    }
}
