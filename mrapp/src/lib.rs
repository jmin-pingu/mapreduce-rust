use plugins_core::{MapFunction, ReduceFunction, PluginRegistrar};
use mr::ds::KeyValue;
use regex::Regex;

pub struct Map;
pub struct Reduce;

plugins_core::export_plugin!(register);

/// 
impl MapFunction for Map {
    fn map(&self, filename: String, contents: String) -> Vec<KeyValue> { 
        let seperator = Regex::new(r"([ \n]+)").expect("Invalid regex");
        let splits: Vec<_> = seperator.split(&contents).into_iter().collect();
        let mut kva: Vec<KeyValue> = vec!();
        for word in splits {
            kva.push(KeyValue::new(word.to_string(), "1".to_string()))
        }
        kva
    } 
}

/// 
impl ReduceFunction for Reduce {
    fn reduce(&self, key: String, values: Vec<String>) -> String { 
        values.len().to_string()
    } 
}

extern "C" fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_map("mapf", Box::new(Map));
    registrar.register_reduce("reducef", Box::new(Reduce));
}

