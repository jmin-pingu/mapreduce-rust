use plugins_core::{MapFunction, ReduceFunction, PluginRegistrar};
use mr::ds::KeyValue;

pub struct Map;
pub struct Reduce;

plugins_core::export_plugin!(register);

impl MapFunction for Map {
    fn map(&self, filename: String, contents: String) -> Vec<KeyValue> {  } 
}

impl ReduceFunction for Reduce {
    fn reduce(&self, key: String, values: Vec<String>) -> String { } 
}

extern "C" fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_map("mapf", Box::new(Map));
    registrar.register_reduce("reducef", Box::new(Reduce));
}


