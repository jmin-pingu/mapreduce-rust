use std::fs;
use std::io;
use std::io::prelude::*;
use std::path;
use mr::ds;
use serde::Deserialize;
use regex::Regex;

#[derive(Deserialize)]
struct Config {
    coordinator: Coordinator, 
    client: Client
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Coordinator {
    address: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Client {
    address: Vec<String>,
    port: Vec<String>
}

fn main() {
    println!("Hello World");

    let contents = fs::read_to_string("config.toml").expect("Should have been able to read the file");    
    let config: Config = toml::from_str( &contents).unwrap();

    println!("{:#?}", config.coordinator);
    println!("{:#?}", config.client);

    let contents = fs::read_to_string("input/pg-being_ernest.txt").expect("Should have been able to read the file");    
    let filename = "input/pg-being_ernest.txt".to_string();

    // create our functions table and load the plugin
    let mut functions = mr::plugins::ExternalFunctions::new();

    unsafe {
        functions
            .load("../target/debug/libplugins_mrapp.so")
            .expect("Function loading failed");
    }

    // then call the function
    let result = functions
        .call_mapf(filename, contents)
        .expect("Invocation failed");

    // print out the result
    println!(
        "mapf: \n{:#?}",
        result
    );

}

async fn check_progress() {

}

