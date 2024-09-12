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

const DELAY: Duration = Duration::from_millis(250);

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
            .load("target/debug/libplugins_mrapp.so")
            .expect("failed to dynamically load map, reduce functions, double-check crate mrapp");
    }

    let flags = Flags::parse();
    let worker: Worker = Worker::new(flags.worker_id, ReduceType::Expedited, flags.nreduce, flags.nmap, flags.server_addr, functions);
    work_until_completion(worker, flags.worker_id).await;
}

async fn work_until_completion(worker: Worker, id: i8) {
    println!("Spawning Worker {}", id);
    let join = task::spawn( async move { 
        loop {
            // TODO: need a condition to exit
            println!("Retry connection");

            match worker.do_work().await {
               MapReduceStatus::Completed => {
                   println!("Worker {} Completed", id);
                   break
               }
               _ => {
                   // let response = worker.send_echo(String::from("hello world")).await;
                   println!("Worker {} In Progress", id);
               }
            }
            thread::sleep(DELAY);
        }
    });
    let val = join.await;
    println!("{:#?}", val);
}

