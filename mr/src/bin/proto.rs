use std::fs;
use std::io;
use std::io::prelude::*;
use std::path;
use mr::ds;
use crate::ds::{
    intermediate::Intermediate,
    MapReduceStatus,
};
use futures::prelude::*;
use plugins_core::ds::KeyValue;
use tarpc::{
    server::{self, Channel},
};
use std::{
    net::SocketAddr,
    hash::{Hasher, Hash, DefaultHasher},
    io::prelude::*, 
    time::Instant,
};
use futures::executor::block_on;

use mr::{
    ds::task::TaskType, 
    rpc::TaskServiceClient
};

use tarpc::{
    client, 
    client::RpcError, 
    context, 
    tokio_serde::formats::Json
};

use serde::Deserialize;
use regex::Regex;
use mr::{
    worker::{ReduceType, Worker},
};
use std::{
    thread,
};
use std::time::Duration;
use clap::Parser;
use tokio::task;

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


#[tokio::main]
pub async fn main() {

    let flags = Flags::parse();
    loop {
        let mut transport = tarpc::serde_transport::tcp::connect(flags.server_addr, Json::default);
        transport.config_mut().max_frame_length(usize::MAX);
        let client: TaskServiceClient = TaskServiceClient::new(client::Config::default(), transport.await.unwrap()).spawn();
        let now = Instant::now();
        let val = client.get_task(context::current(), flags.worker_id, None).await;
        println!("RPC, Task received, took {:#?}", Instant::now() - now);
    }
}

