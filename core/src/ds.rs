use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct KeyValue { 
    pub key: String, 
    pub value: String
}

impl KeyValue {
    pub fn new(key: String, value: String) -> KeyValue {
        KeyValue{ key, value }
    }
} 
