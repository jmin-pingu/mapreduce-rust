use plugins_core::{MapFunction, ReduceFunction, PluginRegistrar};
use plugins_core::ds::KeyValue;

/// ADD NEW MAPREDUCE APPLICATIONS HERE
pub mod wc;

pub struct Map;
pub struct Reduce;

plugins_core::export_plugin!(register);

impl MapFunction for Map {
    fn mapf(&self, filename: String, contents: String) -> Vec<KeyValue> { 
        wc::mapf(filename, contents)
    } 
}

impl ReduceFunction for Reduce {
    fn reducef(&self, key: String, values: Vec<String>) -> String { 
        wc::reducef(key, values)
    } 
}


extern "C" fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_map(Box::new(Map));
    registrar.register_reduce(Box::new(Reduce));
}


