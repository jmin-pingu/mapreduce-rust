use mr::rpc::TaskService;
use tarpc::{
    context,
    server::{self, incoming::Incoming, Channel},
    tokio_serde::formats::Json,
};
use std::time::Instant;
use mr::worker::ReduceType;
use tokio::{
    task,
    sync::Mutex,
};
use futures::{future, prelude::*};
use mr::ds::{
    task::{State, TaskType, Task, TaskID}, 
    taskman::TaskManager,
    MapReduceStatus
};
use clap::Parser;
use std::{
    thread,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    fs,
    time::Duration,
    sync::Arc
};

const TIMEOUT: Duration = Duration::from_secs(10);
const DELAY: Duration = Duration::from_millis(5000);

#[derive(Clone)]
pub struct Server {
    coordref: Arc<Mutex<Coordinator>>,
    socket_addr: SocketAddr
}
// This is the service definition. It looks a lot like a trait definition.

pub struct Coordinator {
    pub taskman: TaskManager, // NOTE: what if we have a daemon looping through the
                                          // task_manager looking for idle map jobs?
}

// NOTE: need to fix Coordinator + Server
impl Coordinator {
    pub fn new(taskman: TaskManager) -> Coordinator {
        Coordinator { taskman }
    }

    pub async fn test(&self) { }

    pub fn check_progress(&mut self, duration: Duration) {
        self.taskman.check_progress(duration);
    }

    pub fn is_complete(&mut self) -> bool {
        self.taskman.status() == MapReduceStatus::Completed
    }

    // Coordinator methods...
    pub async fn get_task(&mut self, id: i8, task_type: Option<TaskType>) -> Option<(Vec<String>, TaskType, Option<i8>)> {
        let now = Instant::now();
        let task_todo = self.taskman.get_idle_task(id, task_type);
        println!("coordinator, Task received, took {:#?}", Instant::now() - now);
        println!("Retrieved task: {:#?}", task_todo);
        task_todo 
    }

    async fn completed_task(&mut self, task: String, reduce_type: ReduceType, nreduce: usize, nmap: usize, worker_id: i8){ 
        println!("Retrieved completed_task RPC for task {}", task);
        self.taskman.task_completed(task, reduce_type, nreduce, nmap, worker_id).unwrap()
    }    

    async fn echo(&self, input: String) -> String {
        format!("{input}")
    }
}

#[derive(Parser)]
struct Flags {
    /// Sets the port number to listen on.
    #[clap(long)]
    port: u16,
    #[clap(long)]
    inputdir: String,
}

// NOTE: enumerate all the input files myself (i.e. give them unique monotonically increasing IDs)

impl mr::rpc::TaskService for Server {
    async fn get_task(self, _: context::Context, id: i8, task_type: Option<TaskType>) -> Option<(Vec<String>, TaskType, Option<i8>)> {
        let coordref = Arc::clone(&self.coordref);
        let mut safe_taskman = coordref.lock().await;
        safe_taskman.get_task(id, task_type).await
    }

    async fn completed_task(self, _: context::Context, task: String, reduce_type: ReduceType, nreduce: usize, nmap: usize, worker_id: i8){ 
        let coordref = Arc::clone(&self.coordref);
        let mut safe_taskman = coordref.lock().await;
        safe_taskman.completed_task(task, reduce_type, nreduce, nmap, worker_id).await;
    }    

    async fn echo(self, _: context::Context, input: String) -> String {
        let coordref = Arc::clone(&self.coordref);
        let safe_taskman = coordref.lock().await;
        safe_taskman.echo(input).await
    }
}

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}

// NOTE: This was the problem
async fn task_checker(coordref: &Arc<Mutex<Coordinator>>) -> tokio::task::JoinHandle<()> {
    let coordref = Arc::clone(coordref);
    let join = task::spawn( async move { 
        loop { 
            let mut safe_taskman = coordref.lock().await;
            if safe_taskman.is_complete() {
                println!("Coordinator shutting down");
                break
            } else {
                println!("Checking Progress");
                safe_taskman.check_progress(TIMEOUT);
            }
            tokio::time::sleep(DELAY).await;
        }
    });
    join
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let flags = Flags::parse();
    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), flags.port);

    // Initialize the TaskManager for the Coordinator
    let mut taskman = TaskManager::new();
    let paths = fs::read_dir(flags.inputdir).unwrap();
    paths.enumerate().for_each(|(i, path)| {
        let task = Task::new(
            vec!(format!("{}", path.unwrap().path().display())),
            TaskType::Map,
            TaskID::MapID(i as i8),
        );
        (&mut taskman).add_task(task);
    });

    println!("{:#?}", taskman);
    let coordinator = Coordinator::new(taskman);
    let coordref = Arc::new(Mutex::new(coordinator));

    // Background routine
    // let join = task_checker(&coordref);
    // println!("Starting server");
    // join.await;

    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;
    tracing::info!("Listening on port {}", listener.local_addr().port());
    listener.config_mut().max_frame_length(usize::MAX);
    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        // Limit channels to 1 per IP.
        .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap().ip())
        // serve is generated by the service attribute. It takes as input any type implementing
        // the generated TaskService trait.
        .map(|channel| {
            let server = Server {coordref: Arc::clone(&coordref), socket_addr: channel.transport().peer_addr().unwrap()};
            channel.execute(server.serve()).for_each(spawn)
        })
        // Max 10 channels.
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;

    Ok(())
}

// aggregate_outputf: combine the completed outputf into one file
// fn aggregate_outputf() { }
