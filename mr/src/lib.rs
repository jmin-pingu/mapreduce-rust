// Modules
use plugins_core::ds::KeyValue;
use libloading::Library;
use plugins_core::{MapFunction, ReduceFunction, PluginDeclaration};
use std::{
    alloc::System, collections::HashMap, ffi::OsStr, io,
    rc::Rc,
};

#[global_allocator]
static ALLOCATOR: System = System;

pub mod ds;
pub mod rpc;
pub mod wc;

/// TODO: double-check all source code below
// Set-up dynamic loading
pub struct MapProxy {
    function: Box<dyn MapFunction>,
    _lib: Rc<Library>
}

pub struct ReduceProxy {
    function: Box<dyn ReduceFunction>,
    _lib: Rc<Library>
}

impl MapFunction for MapProxy {
    fn mapf(&self, filename: String, contents: String) -> Vec<KeyValue> { 
        self.function.mapf(filename, contents)
    } 
}

impl ReduceFunction for ReduceProxy {
    fn reducef(&self, key: String, values: Vec<String>) -> String {
        self.function.reducef(key, values)
    } 
}

enum FunctionProxy {
    Map(MapProxy),
    Reduce(ReduceProxy),
}

#[derive(Default)]
pub struct ExternalFunctions {
    functions: HashMap<String, FunctionProxy>,
    libraries: Vec<Rc<Library>>,
}

impl ExternalFunctions {
    pub fn new() -> ExternalFunctions {
        ExternalFunctions::default()
    }

    /// TODO: specifically, double-check this segment
    pub unsafe fn load<P: AsRef<OsStr>>(&mut self, library_path: P) -> io::Result<()> {
        let library = Rc::new(Library::new(library_path).unwrap());

        // get a pointer to the plugin_declaration symbol.
        let decl = library
            .get::<*mut PluginDeclaration>(b"plugin_declaration\0")
            .unwrap()
            .read();

        // version checks to prevent accidental ABI incompatibilities
        if decl.rustc_version != plugins_core::RUSTC_VERSION
            || decl.core_version != plugins_core::CORE_VERSION
        {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Version mismatch",
            ));
        }

        let mut registrar = PluginRegistrar::new(Rc::clone(&library));

        (decl.register)(&mut registrar);

        // add all loaded plugins to the functions map
        self.functions.extend(registrar.functions);
        // and make sure ExternalFunctions keeps a reference to the library
        self.libraries.push(library);

        Ok(())
    }

    /// Call Map by Name
    pub fn call_mapf(&self, filename: String, contents: String) -> Result<Vec<KeyValue>, plugins_core::InvocationError> {
        if let FunctionProxy::Map(func) = self.functions
            .get("mapf")
            .ok_or_else(|| format!("\"mapf\" not found"))? {

            Ok(func.mapf(filename, contents))
        } else {
            panic!("call to mapf failed")
        }
    }

    pub fn call_reducef(&self, key: String, values: Vec<String>) -> Result<String, plugins_core::InvocationError> {
        if let FunctionProxy::Reduce(func) = self.functions
            .get("mapf")
            .ok_or_else(|| format!("\"reducef\" not found"))? 
        {
            Ok(func.reducef(key, values))
        } else {
            panic!("call to mapf failed")
        }
    }
}

struct PluginRegistrar {
    functions: HashMap<String, FunctionProxy>,
    lib: Rc<Library>,
}

impl PluginRegistrar {
    fn new(lib: Rc<Library>) -> PluginRegistrar {
        PluginRegistrar {
            lib,
            functions: HashMap::default(),
        }
    }
}

impl plugins_core::PluginRegistrar for PluginRegistrar {
    fn register_map(&mut self, function: Box<dyn MapFunction>) {
        let proxy = MapProxy {
            function,
            _lib: Rc::clone(&self.lib),
        };
        self.functions.insert("mapf".to_string(), FunctionProxy::Map(proxy));
    }


    fn register_reduce(&mut self, function: Box<dyn ReduceFunction>) { 
        let proxy = ReduceProxy {
            function,
            _lib: Rc::clone(&self.lib),
        };
        self.functions.insert("reducef".to_string(), FunctionProxy::Reduce(proxy));
    }
}
