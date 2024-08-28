use std::collections::HashMap;
use std::time::Instant;
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Debug)]
pub struct Task { // How do I want to use this in a meaningful way 
    task: String, // NOTE: path to map file or reduce file 
    worker: i8,
    state: State,
    task_type: TaskType, 
}

#[derive(Debug)]
pub struct TimedTask {
    task: Task,
    started: Instant,
}

#[derive(Debug)]
pub enum TaskType {
    Map,
    Reduce,
}

#[derive(Debug)]
pub enum State {
    InProgress,
    Idle,
    Completed,
}

/// A thread safe data structure which keeps track of when a task is started 
/// and supports task management operations.
#[derive(Debug)]
pub struct TaskManager {
    map: Arc<Mutex<HashMap<String, TimedTask>>>
}
 
impl TaskManager {
    /// Returns an empty task manager
    fn new() -> Self {
        TaskManager{ 
            map: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    /// Adds a task to the manager
    ///
    /// # Arguments
    ///
    /// * `task`
    pub fn add_task(&mut self, task: Task) {
        let map_ref: Arc<Mutex<HashMap<String, TimedTask>>> = Arc::clone(&self.map);
        let mut map: MutexGuard<'_, HashMap<String, TimedTask>> = map_ref.lock().unwrap();
        (*map).insert(task.task.clone(), TimedTask{ task, started:Instant::now()});
    }

    /// DESCRIPTION
    ///
    /// # Arguments
    ///
    /// * `task`
    pub fn get_task(&self, task: Task) -> TimedTask {
        let map_ref: Arc<Mutex<HashMap<String, TimedTask>>> = Arc::clone(&self.map);
        let map: MutexGuard<'_, HashMap<String, TimedTask>> = map_ref.lock().unwrap();
        (*map).get(&task.task).unwrap()
    }

    /// DESCRIPTION
    pub fn get_size(&self) -> usize {
        let map_ref: Arc<Mutex<HashMap<String, TimedTask>>> = Arc::clone(&self.map);
        let map: MutexGuard<'_, HashMap<String, TimedTask>> = map_ref.lock().unwrap();
        (*map).len()
    }
}

