use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Debug)]
pub struct Task { 
    path: String, 
    worker: Option<i8>,
    state: State,
    task_type: TaskType, 
}

/// A Task contains information about tasks that a Worker will be assigned to complete by the
/// Coordinator
impl Task {
    pub fn new(path: String, state: State, task_type: TaskType) -> Self { 
        Task {
            path, 
            worker: None, 
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

    fn get_path(&self) -> String { 
        self.path.clone()
    }

    fn get_task_type(&self) -> TaskType { 
        self.task_type.clone()
    }

    fn set_worker(&mut self, worker: i8) { 
        self.worker = Some(worker);
    }
}

#[derive(Debug)]
/// A TimedTask is a Task with an associated Instant, which represents when the Task was started
pub struct TimedTask {
    task: Task,
    started: Instant,
}

impl TimedTask {
    /// Creates a TimedTask
    ///
    /// # Arguments
    ///
    /// * `task`: the task which will now have an associated start time
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

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
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
    list: Arc<Mutex<Vec<TimedTask>>>
}
 
impl TaskManager {
    /// Returns an empty task manager
    pub fn new() -> Self {
        TaskManager{ 
            list: Arc::new(Mutex::new(Vec::new()))
        }
    }

    /// Adds a task to the manager
    ///
    /// # Arguments
    ///
    /// * `task`
    pub fn add_task(&mut self, task: Task) {
        let taskman_ref: Arc<Mutex<Vec<TimedTask>>> = Arc::clone(&self.list);
        let mut taskman: MutexGuard<'_, Vec<TimedTask>> = taskman_ref.lock().unwrap();
        (*taskman).push(TimedTask::new(task));
    }

    /// Get the first available Idle task to give to a worker
    ///
    pub fn get_task(&mut self, id: i8, task_type: Option<TaskType>) -> Option<String> {
        let taskman_ref: Arc<Mutex<Vec<TimedTask>>> = Arc::clone(&self.list);
        let mut taskman: MutexGuard<'_, Vec<TimedTask>> = taskman_ref.lock().unwrap();
        // NOTE: Potential concern is that we are modifying a list as we are iterating over it.
        for timed_task in &mut (*taskman) {
            if timed_task.task.get_state() == State::Idle && (task_type == Some(timed_task.task.get_task_type()) || task_type == None){
                timed_task.task.set_worker(id);
                return Some(timed_task.task.get_path())
            }         
        }
        None
    }

    pub fn check_task_type(&self) -> bool { true } 

    /// Remove completed tasks 
    pub fn clean(&mut self) {
        let taskman_ref: Arc<Mutex<Vec<TimedTask>>> = Arc::clone(&self.list);
        let mut taskman: MutexGuard<'_, Vec<TimedTask>> = taskman_ref.lock().unwrap();
        (*taskman).retain(|timed_task| timed_task.task.get_state() != State::Completed);
    }

    /// Get the size of the list of tasks remaining 
    pub fn get_size(&self) -> usize {
        let taskman_ref: Arc<Mutex<Vec<TimedTask>>> = Arc::clone(&self.list);
        let taskman: MutexGuard<'_, Vec<TimedTask>> = taskman_ref.lock().unwrap();
        (*taskman).len()
    }
}

