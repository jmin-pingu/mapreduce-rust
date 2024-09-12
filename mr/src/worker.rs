use crate::ds::{
    intermediate::Intermediate,
    MapReduceStatus,
};
use crate::plugins::ExternalFunctions;
use plugins_core::ds::KeyValue;
use std::{
    net::SocketAddr,
    hash::{Hasher, Hash, DefaultHasher},
    fs,
    io::prelude::*, 
    path, 
    io,
};

use crate::{
    ds::task::TaskType, 
    rpc::TaskServiceClient,
};

use tarpc::{
    client, 
    client::RpcError, 
    context, 
    tokio_serde::formats::Json
};

// How do I implement "waiting for all maps to be completed"
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum ReduceType {
    Expedited,
    Traditional,
}

pub struct Worker {
    worker_id: i8,    
    reduce_type: ReduceType,
    nreduce: usize,
    nmap: usize,
    server_addr: SocketAddr,
    functions: ExternalFunctions
    // NOTE: do I want to add additional metadata?
}

impl Worker {
    pub fn new(worker_id: i8, reduce_type: ReduceType, nreduce: usize, nmap: usize, server_addr: SocketAddr, functions: ExternalFunctions) -> Self {
        Worker {
            worker_id, 
            reduce_type,
            nreduce, 
            nmap,
            server_addr,
            functions
        }
    }

    pub fn get_id(&self) -> i8 {
        self.worker_id
    }

    pub async fn do_work(&self) -> Result<MapReduceStatus, RpcError> {
        // TODO: depending on ReduceType, eagerly get reduce tasks when available or wait for no map tasks 
        match self.reduce_type {
            ReduceType::Expedited => {
                // Get whatever task is available.
                println!("Reduce Type Expedited");
                let task = self.send_get_task(None).await;
                match task {
                    Ok(None) => { Ok(MapReduceStatus::Completed) }
                    Ok(Some((paths, task_type, task_id))) => {
                        match task_type {
                            // TODO: double-check logic
                            TaskType::Map => { 
                                let echo = self.send_echo(String::from("Hello World")).await;
                                println!("{:#?}", echo);
                                for filename in paths {
                                    let kva = self.do_map(filename.clone());
                                    self.prepare_for_reduce(task_id.unwrap() as usize, kva);

                                    let echo = self.send_echo(String::from("Hello World")).await;
                                    println!("{:#?}", echo);
                                }
                            }
                            TaskType::Reduce => { 
                                for filename in paths {
                                    self.do_reduce(filename);
                                }
                            } 
                        }
                        Ok(MapReduceStatus::InProgress)
                    }
                }
            }
            //  fn do_map(filename: String, functions: &ExternalFunctions) -> Vec<KeyValue>
            //  fn do_reduce(filename: String, functions: &ExternalFunctions)
            //  fn prepare_for_reduce(map_task_num: usize, kva: Vec<KeyValue>)
            ReduceType::Traditional => {
                println!("Reduce Type Traditional");
                let map_task = self.send_get_task(Some(TaskType::Map)).await.unwrap();
                // Get whatever task is available.
                match map_task {
                    Some((paths, _, task_id)) => {
                        for filename in paths {
                            let kva = self.do_map(filename);
                            self.prepare_for_reduce(task_id.unwrap() as usize, kva);
                        }
                        MapReduceStatus::InProgress
                    }
                    None => {
                        let reduce_task = self.send_get_task(Some(TaskType::Reduce)).await.unwrap();
                        match reduce_task {
                            Some((paths, _, _)) => {
                                for filename in paths {
                                    self.do_reduce(filename);
                                }
                                MapReduceStatus::InProgress
                            }
                            None => {
                                MapReduceStatus::Completed
                            }
                        }
                    }
                }
            }
        }
    }
     


    /// Description
    ///
    /// Arguments
    fn do_map(&self, filename: String) -> Vec<KeyValue> {
        println!("Doing map for filename {}", filename);
        let (filename, contents) = self.read_file(filename);
        self.functions
            .call_mapf(filename, contents)
            .expect("Invocation failed")
    }

    /// Description
    ///
    /// Arguments
    ///
    // TODO: come up with a better name for preparing the output KeyValue's of mapf for nreduce reducef
    // tasks
    fn prepare_for_reduce(&self, map_task_num: usize, kva: Vec<KeyValue>) {
        // Initialize the vector of nreduce temporary files
        let mut files: Vec<fs::File> = Vec::with_capacity(self.nreduce);
        for reduce_task_num in 0..self.nreduce {
            let file = fs::File::create(format!("{}/mr-{}-{}", std::env::current_dir().unwrap().into_os_string().into_string().unwrap(), map_task_num, reduce_task_num)).unwrap();
            files.push(file);
        }
    
        // Add all KeyValue's to appropriate intermediate files
        kva.into_iter().for_each(|kv| {
            let data = format!("{}\n", serde_json::to_string(&kv).unwrap());
            (&files[self.hash::<String>(&kv.key) as usize % self.nreduce])
                .write(data.as_bytes())
                .expect("Unable to write to intermediate file"); 
        });
    }

    pub fn do_reduce(&self, filename: String) {
        let mut intermediate: Intermediate = Intermediate::new();
    
        // Get the reduce task number from the filename, DO NOT move the filename since we still need
        // to read lines from the filename
        match self.reduce_type {
            ReduceType::Expedited => { }
            ReduceType::Traditional => { }
        }
    
        let re = regex::Regex::new(r"mr-[0-9]+-(?<reduce_num>[0-9]+)").expect("regex pattern invalid");
        let capture_group = re.captures(&filename).expect("filename is incorrect");
        let reduce_task_num = (&capture_group)["reduce_num"].to_string();
        // Read lines from file into Intermediate
        if let Ok(lines) = self.read_lines(filename.clone()) {
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
        intermediate.0 // TODO: impl Iterator for Intermediate
            .into_iter()
            .for_each(|(k, v)| {
                outputf.write(
                    format!("{} {}\n", 
                        k.clone(), 
                        self.functions
                            .call_reducef(k, v)
                            .expect("Invocation failed")
                        ).as_bytes()
                    ).expect("Failed to write to output file");
                }
            );
        // Delete the intermediate filename
        fs::remove_file(filename).expect("Failed to remove filename");
    }

    // The output is wrapped in a Result to allow matching on errors.
    // Source: https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
    fn read_lines<P>(&self, filename: P) -> io::Result<io::Lines<io::BufReader<fs::File>>>
    where P: AsRef<path::Path>, {
        let file = fs::File::open(filename).expect("Failed to read filename");
        Ok(io::BufReader::new(file).lines())
    }

    /// read_file: 
    fn read_file(&self, file_name: String) -> (String, String) {
        let contents = fs::read_to_string(file_name.clone()).expect(format!("Should have been able to read file {:#?}", file_name.clone()).as_str());
        (file_name, contents)
    }

    /// hash: Calculates the hash for a generic T that implements Hash
    fn hash<T: Hash>(&self, t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    /// Define client-side RPC calls
    pub async fn send_completed_task(&self, task: String, worker_id: i8) -> Result<(), RpcError> {
        let mut transport = tarpc::serde_transport::tcp::connect(self.server_addr, Json::default);
        transport.config_mut().max_frame_length(usize::MAX);
        let client: TaskServiceClient = TaskServiceClient::new(client::Config::default(), transport.await.unwrap()).spawn();
        println!("Sending completed_task RPC");
        client.completed_task(context::current(), task, self.reduce_type.clone(), self.nreduce, self.nmap, worker_id).await
    }

    pub async fn send_get_task(&self, task_type: Option<TaskType>) -> Result<Option<(Vec<String>, TaskType, Option<i8>)>, RpcError> {
        let mut transport = tarpc::serde_transport::tcp::connect(self.server_addr, Json::default);
    
        transport.config_mut().max_frame_length(usize::MAX);
        let client: TaskServiceClient = TaskServiceClient::new(client::Config::default(), transport.await.unwrap()).spawn();
        client.get_task(context::current(), self.worker_id, task_type).await
    }

    pub async fn send_echo(&self, arg: String) -> Result<String, RpcError> {
        let mut transport = tarpc::serde_transport::tcp::connect(self.server_addr, Json::default);
    
        transport.config_mut().max_frame_length(usize::MAX);
        // TaskServiceClient is generated by the #[tarpc::service] attribute. It has a constructor `new`
        // that takes a config and any Transport as input.
        let client: TaskServiceClient = TaskServiceClient::new(client::Config::default(), transport.await.unwrap()).spawn();
    
        // The client has an RPC method for each RPC defined in the annotated trait. It takes the same
        // args as defined, with the addition of a Context, which is always the first arg. The Context
        // specifies a deadline and trace information which can be helpful in debugging requests.
        client.echo(context::current(), arg).await
    }

}

