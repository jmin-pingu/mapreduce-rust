use mr::worker::{ReduceType, Worker};
use std::{
    net::SocketAddr,
    thread,
};
use clap::Parser;

#[derive(Parser)]
struct Flags {
    #[clap(long)]
    worker_id: i8,
    
    #[clap(long)]
    nmap: usize,

    #[clap(long)]
    nreduce: usize,

    #[clap(long)]
    reduce_type: Option<String>,

    /// Sets the SERVER address to connect to.
    #[clap(long)]
    server_addr: SocketAddr,
}

#[tokio::main]
pub async fn main() {
    let mut functions = mr::plugins::ExternalFunctions::new();

    unsafe {
        functions
            .load("../target/debug/libplugins_mrapp.so")
            .expect("Function loading failed");
    }

    let x: String = "Hello World".to_string();
    let flags = Flags::parse();
    let worker: Worker = create_worker(flags.worker_id, ReduceType::Expedited, flags.nreduce, flags.nmap, flags.server_addr);
     
    let handle = thread::spawn(move ||
        worker.send_echo(x)
    );

    let result = handle.join().unwrap().unwrap();
    println!("{}", result);

}

/// Description
///
/// Arguments
pub fn create_worker(
    id: i8, 
    reduce_type: ReduceType,
    nreduce: usize,
    nmap: usize,
    server_addr: SocketAddr
) -> Worker {
    // create our functions table and load the plugin

    Worker::new(id, reduce_type, nreduce, nmap, server_addr)
}

