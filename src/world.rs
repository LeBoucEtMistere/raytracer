use super::collision::{Hittable, HittableList};
use std::sync::Arc;
pub struct World {
    hittables: Arc<HittableList>,
}

pub struct WorldBuilder {
    hittables: HittableList,
}

impl WorldBuilder {
    pub fn add_object(mut self, object: impl Hittable + 'static) -> Self {
        let hittable: Arc<dyn Hittable> = Arc::new(object) as Arc<dyn Hittable>;
        self.hittables.add_hittable(hittable);
        self
    }

    pub fn build(self) -> World {
        World {
            hittables: Arc::new(self.hittables),
        }
    }
}

impl World {
    pub fn builder() -> WorldBuilder {
        WorldBuilder {
            hittables: HittableList::new(),
        }
    }

    pub fn get_hittables(&self) -> Arc<HittableList> {
        Arc::clone(&self.hittables)
    }
}
