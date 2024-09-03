// Much of the plugin code is inspired by this blog post by Michael Bryan: https://adventures.michaelfbryan.com/posts/plugins-in-rust/
use mr::ds::KeyValue;

pub static CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub static RUSTC_VERSION: &str = env!("RUSTC_VERSION");

pub trait MapFunction {
    fn map(&self, filename: String, contents: String) -> Vec<KeyValue>;
}

pub trait ReduceFunction {
    fn reduce(&self, key: String, values: Vec<String>) -> String;
}

pub enum InvocationError {
    InvalidArgumentCount { expected: usize, found: usize }, 
    Other { msg: String },
}

pub struct PluginDeclaration {
    pub rustc_version: &'static str,
    pub core_version: &'static str,
    pub register: unsafe extern "C" fn(&mut dyn PluginRegistrar),
}

pub trait PluginRegistrar {
    fn register_map(&mut self, name: &str, function: Box<dyn MapFunction>);

    fn register_reduce(&mut self, name: &str, function: Box<dyn ReduceFunction>);
}

#[macro_export]
macro_rules! export_plugin {
    ($register:expr) => {
        #[doc(hidden)]
        #[no_mangle]
        pub static plugin_declaration: $crate::PluginDeclaration = $crate::PluginDeclaration {
            rustc_version: $crate::RUSTC_VERSION,
            core_version: $crate::CORE_VERSION,
            register: $register,
        };
    };
}
