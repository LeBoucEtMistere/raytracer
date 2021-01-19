use super::material::{Diffuse, Material};
use std::collections::HashMap;
use std::sync::Arc;

pub struct MaterialAtlas {
    atlas: HashMap<String, Arc<Box<dyn Material>>>,
}

impl Default for MaterialAtlas {
    fn default() -> Self {
        let mut atlas = HashMap::new();
        atlas.insert(
            String::from("Default"),
            Arc::new(Box::new(Diffuse::default()) as Box<dyn Material>),
        );
        MaterialAtlas { atlas }
    }
}

impl MaterialAtlas {
    pub fn insert_material(
        &mut self,
        name: &str,
        object: impl Material + 'static,
    ) -> Option<Arc<Box<dyn Material>>> {
        let material: Arc<Box<dyn Material>> = Arc::new(Box::new(object) as Box<dyn Material>);
        self.atlas.insert(String::from(name), material)
    }

    pub fn get_material(&self, name: &str) -> Option<Arc<Box<dyn Material>>> {
        self.atlas.get(name).map(|x| Arc::clone(x))
    }
}
