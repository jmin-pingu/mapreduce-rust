use std::collections::HashMap;
use hashbrown::hash_map as base;

/// DESC
pub struct Intermediate(pub HashMap<String, Vec<String>>);


/// DESC
impl Intermediate {
    pub fn new() -> Self {
        Intermediate(HashMap::new())
    }

    // TODO: Is there a way to make this not OWNED (i.e. &str)
    pub fn insert(&mut self, key: String, value: String) {
        if let Some(values) = self.0.get_mut(&key) {
            values.push(value);
        } else {
            self.0.insert(key, vec![value]);
        }
    }

    // TODO: Is there a way to make this not OWNED (i.e. &str)
    pub fn get(&mut self, key: String) -> Option<&Vec<String>> {
        self.0.get(&key) 
    }
}
