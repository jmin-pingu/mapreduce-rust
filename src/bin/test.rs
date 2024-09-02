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
    let seperator = Regex::new(r"([ \n]+)").expect("Invalid regex");
    let splits: Vec<_> = seperator.split(&contents).into_iter().collect();
    println!("{:#?}", splits);
}

fn mapf() { }

fn reducef() { }
