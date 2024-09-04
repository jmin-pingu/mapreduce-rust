use std::{
    time::{Instant, Duration},
    sync::{Arc, Mutex, MutexGuard}
};
use super::task::{Task, TaskType, State};

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

/// A thread safe data structure which keeps track of when a task is started 
/// and supports task management operations.
#[derive(Debug, Clone)]
pub struct TaskManager {
    list: Arc<Mutex<Vec<TimedTask>>>
}
 
impl TaskManager {
    /// new: 
    ///
    pub fn new() -> Self {
        TaskManager{ 
            list: Arc::new(Mutex::new(Vec::new()))
        }
    }

    /// add_task: Adds a task to the manager
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
    // NOTE: Think of a better name for this. 
    pub fn get_idle_task(&mut self, id: i8, task_type: Option<TaskType>) -> Option<String> {
        let taskman_ref: Arc<Mutex<Vec<TimedTask>>> = Arc::clone(&self.list);
        let mut taskman: MutexGuard<'_, Vec<TimedTask>> = taskman_ref.lock().unwrap();
        for timed_task in &mut (*taskman) {
            if timed_task.task.get_state() == State::Idle && (task_type == Some(timed_task.task.get_task_type()) || task_type == None){ // NOTE: double check logic of second logical statement
                timed_task.task.set_worker(id);
                return Some(timed_task.task.get_path())
            }         
        }
        None
    }

    /// update_state: Update the state of a task if it exists (return true)
    pub fn update_state(&mut self, task: String, state: State) -> bool {
        let taskman_ref: Arc<Mutex<Vec<TimedTask>>> = Arc::clone(&self.list);
        let mut taskman: MutexGuard<'_, Vec<TimedTask>> = taskman_ref.lock().unwrap();
        for timed_task in &mut (*taskman) {
            if timed_task.task.get_path() == task {
                timed_task.task.set_state(state);
                return true
            }         
        }
        false
    }

    // pub fn check_task_type(&self) -> bool { } 

    /// clean: Remove completed tasks 
    pub fn clean(&mut self) {
        let taskman_ref: Arc<Mutex<Vec<TimedTask>>> = Arc::clone(&self.list);
        let mut taskman: MutexGuard<'_, Vec<TimedTask>> = taskman_ref.lock().unwrap();
        (*taskman).retain(|timed_task| timed_task.task.get_state() != State::Completed);
    }

    /// get_size: Get the size of the list of tasks remaining 
    pub fn get_size(&self) -> usize {
        let taskman_ref: Arc<Mutex<Vec<TimedTask>>> = Arc::clone(&self.list);
        let taskman: MutexGuard<'_, Vec<TimedTask>> = taskman_ref.lock().unwrap();
        (*taskman).len()
    }
}

