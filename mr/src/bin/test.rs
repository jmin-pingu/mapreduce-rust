use std::fs;
use std::io;
use std::env;
use std::io::prelude::*;
use std::path;
use mr::ds;
use plugins_core::ds::KeyValue;
use serde::Deserialize;
use regex::Regex;

#[derive(Deserialize)]
struct Config {
    coordinator: Coordinator, 
    workers: Workers
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Coordinator {
    address: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Workers {
    address: Vec<String>,
    port: Vec<String>
}

fn main() {
    println!("Hello World");

    let contents = fs::read_to_string("../config.toml").expect("Should have been able to read the file");    
    let config: Config = toml::from_str( &contents).unwrap();

    println!("{:#?}", config.coordinator);
    println!("{:#?}", config.workers);

    let path = env::current_dir().unwrap();
    println!("The current directory is {}", path.display());

    let contents = fs::read_to_string("../input/pg-being_ernest.txt").expect("Should have been able to read the file");    
    let filename = "../input/pg-being_ernest.txt".to_string();

    // create our functions table and load the plugin
    let mut functions = mr::ExternalFunctions::new();

    unsafe {
        functions
            .load("../target/debug/libplugins_mrapp.so")
            .expect("Function loading failed");
    }

    // NOTE: dynamically loaded
    let result = functions
        .call_mapf(filename, contents)
        .expect("Invocation failed");

    // NOTE: non-dynamically loaded
    // let result = mapf(filename, contents);
    // print out the result
    println!(
        "mapf output: {:#?}",
        result
    );

}

fn mapf(filename: String, contents: String) -> Vec<KeyValue> { 
    let seperator = Regex::new(r"([ \n]+)").expect("Invalid regex");
    let splits: Vec<_> = seperator.split(&contents).into_iter().collect();
    let mut kva: Vec<KeyValue> = vec!();
    for word in splits {
        kva.push(KeyValue::new(word.to_string(), "1".to_string()))
    }
    kva
} 
