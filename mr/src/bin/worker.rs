use mr::{
    worker::{ReduceType, Worker},
    ds::MapReduceStatus
};
use std::{
    net::SocketAddr,
    thread,
};
use std::time::Duration;
use clap::Parser;
use tokio::task;

const DELAY: Duration = Duration::from_millis(500);

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
            .expect("failed to dynamically load map, reduce functions, double-check crate mrapp");
    }

    let flags = Flags::parse();
    let worker: Worker = create_worker(flags.worker_id, ReduceType::Expedited, flags.nreduce, flags.nmap, flags.server_addr);
     
    let join = task::spawn( async move { 
        loop { 
            // TODO: need a condition to exit
            match worker.do_work() {
               MapReduceStatus::Completed => { 
                   println!("Worker {} Completed", flags.worker_id);
                   break 
               }
               _ => {}
            }
            thread::sleep(DELAY);
        }
    });
    join.await.unwrap();
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

