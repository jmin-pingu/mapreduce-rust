use std::collections::HashMap;
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Debug)]
pub struct Task { // How do I want to use this in a meaningful way 
    task: String, // NOTE: path to map file or reduce file 
    worker: i8,
    state: State,
    task_type: TaskType, 
}

impl Task {
    pub fn new(task: String, worker: i8, state: State, task_type: TaskType) -> Self { 
        Task {
            task, 
            worker, 
            state,
            task_type
        }
    }

    fn set_state(&mut self, state: State) { 
        self.state = state;
    }

    fn get_state(&self) -> State { 
        self.state.clone()
    }

    fn set_worker(&mut self, worker: i8) { 
        self.worker = worker;
    }
}


#[derive(Debug)]
pub struct TimedTask {
    task: Task,
    started: Instant,
}

impl TimedTask {
    fn new(task: Task) -> TimedTask {
        TimedTask {
            task, 
            started: Instant::now()
        }
    }

    /// Checks the progress of a TimedTask and changes the state of the task to State::Idle
    /// if it is taking "too long" for the task to complete
    ///
    /// # Arguments
    ///
    /// * `duration`: the upper bound duration that the task should take to complete
    pub fn check_progress(&mut self, duration: Duration) {
        if self.task.get_state() == State::InProgress && Instant::now() - self.started >= duration {
            self.task.set_state(State::Idle)
        }
    }
}

#[derive(Debug)]
pub enum TaskType {
    Map,
    Reduce,
}

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    InProgress,
    Idle,
    Completed,
}

/// A thread safe data structure which keeps track of when a task is started 
/// and supports task management operations.
#[derive(Debug, Clone)]
pub struct TaskManager {
    map: Arc<Mutex<HashMap<String, TimedTask>>>
}
 
impl TaskManager {
    /// Returns an empty task manager
    pub fn new() -> Self {
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
        (*map).insert(task.task.clone(), TimedTask::new(task));
    }

    /// Get the first available Idle task to give to a worker
    pub fn get_task(&mut self, id: i8) -> Option<String> {
        let map_ref: Arc<Mutex<HashMap<String, TimedTask>>> = Arc::clone(&self.map);
        let mut map: MutexGuard<'_, HashMap<String, TimedTask>> = map_ref.lock().unwrap();
        // NOTE: Potential concern is that we are modifying a list as we are iterating over it.
        for (key, timed_task) in &mut (*map) {
            if timed_task.task.state == State::Idle {
                timed_task.task.set_worker(id);
                return Some(key.clone())
            } else if timed_task.task.state == State::Completed {
                self.remove_task(timed_task.task.task.clone());
            }
        }
        None
    }

    /// Remove the TimedTask
    pub fn remove_task(&mut self, task: String) {
        let map_ref: Arc<Mutex<HashMap<String, TimedTask>>> = Arc::clone(&self.map);
        let mut map: MutexGuard<'_, HashMap<String, TimedTask>> = map_ref.lock().unwrap();
        (*map).remove(&task);
    }

    /// DESCRIPTION
    pub fn get_size(&self) -> usize {
        let map_ref: Arc<Mutex<HashMap<String, TimedTask>>> = Arc::clone(&self.map);
        let map: MutexGuard<'_, HashMap<String, TimedTask>> = map_ref.lock().unwrap();
        (*map).len()
    }
}

