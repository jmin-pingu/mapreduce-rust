use std::fs;
use mr::ds::{
    task::TaskType, 
    intermediate::Intermediate
};
use plugins_core::ds::KeyValue;
use serde_json;
use std::{
    thread,
    io::prelude::*, 
    path, 
    io,
};
use std::hash::{Hasher, Hash, DefaultHasher};
use std::env;
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
    format!("{}", path.display());

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

    prepare_for_reduce(1, 5, result);
    println!(
        "finished preparing for reduce",
    );

    do_reduce("mr-1-0".to_string(), &functions);
    do_reduce("mr-1-1".to_string(), &functions);
    do_reduce("mr-1-2".to_string(), &functions);
    do_reduce("mr-1-3".to_string(), &functions);
    do_reduce("mr-1-4".to_string(), &functions);
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

/// hash: Calculates the hash for a generic T that implements Hash
fn hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn prepare_for_reduce(map_task_num: usize, nreduce: usize, kva: Vec<KeyValue>) {
    // Initialize the vector of nreduce temporary files
    let mut files: Vec<fs::File> = Vec::with_capacity(nreduce);
    println!("{}", format!("{}/mr-{}-{}", format!("{}", std::env::current_dir().unwrap().display()), map_task_num, 0));

    for reduce_task_num in 0..nreduce {
        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(format!("{}/mr-{}-{}", format!("{}", std::env::current_dir().unwrap().display()), map_task_num, reduce_task_num)).unwrap();
        files.push(file);
    }

    // Add all KeyValue's to appropriate intermediate files
    kva.into_iter().for_each(|kv| {
        let data = format!("{}\n", serde_json::to_string(&kv).unwrap());
        (&files[hash::<String>(&kv.key) as usize % nreduce])
            .write(data.as_bytes())
            .expect("Unable to write to intermediate file"); 
    });
}

fn do_reduce(filename: String, functions: &mr::ExternalFunctions) {
    let mut intermediate: Intermediate = Intermediate::new();

    // Get the reduce task number from the filename, DO NOT move the filename since we still need
    // to read lines from the filename
    let re = regex::Regex::new(r"mr-[0-9]+-(?<reduce_num>[0-9]+)").expect("regex pattern invalid");
    let capture_group = re.captures(&filename).expect("filename is incorrect");
    let reduce_task_num = (&capture_group)["reduce_num"].to_string();

    // Read lines from file into Intermediate
    if let Ok(lines) = read_lines(filename.clone()) {
        lines.flatten().for_each(|line| {
            let kv: KeyValue = serde_json::from_str(&line).unwrap();
            intermediate.insert(kv.key, kv.value);
        });    
    };

    // Open reducef (create if it doesn't exist, append if it does)
    let mut outputf = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(format!("mr-out-{}", reduce_task_num))
        .expect("Failed to open output reduce file");

    // Apply reducef to the values in intermediate to a file
    println!("Applying reducef to values in intermediate file");
    intermediate.0 // TODO: impl Iterator for Intermediate
        .into_iter()
        .for_each(|(k, v)| {
            outputf.write(
                format!("{} {}\n", 
                    k.clone(), 
                    functions
                        .call_reducef(k, v)
                        .expect("Invocation failed")
                    ).as_bytes()
                ).expect("Failed to write to output file");
            }
        );
    // Delete the intermediate filename
    fs::remove_file(filename).expect("Failed to remove filename");
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<fs::File>>>
where P: AsRef<path::Path>, {
    let file = fs::File::open(filename).expect("Failed to read filename");
    Ok(io::BufReader::new(file).lines())
}
