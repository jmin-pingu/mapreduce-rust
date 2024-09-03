use plugins_core::{MapFunction, ReduceFunction, PluginRegistrar};
use plugins_core::ds::KeyValue;

pub struct Map;
pub struct Reduce;

plugins_core::export_plugin!(register);

impl MapFunction for Map {
    fn mapf(&self, filename: String, contents: String) -> Vec<KeyValue> {  } 
}

impl ReduceFunction for Reduce {
    fn reducef(&self, key: String, values: Vec<String>) -> String { } 
}

extern "C" fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_map("mapf", Box::new(Map));
    registrar.register_reduce("reducef", Box::new(Reduce));
}


