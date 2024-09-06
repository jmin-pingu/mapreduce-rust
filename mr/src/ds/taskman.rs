use std::{
    time::{Instant, Duration},
    sync::{Arc, Mutex, MutexGuard},
    fmt
};
use crate::worker::ReduceType;
use super::task::{Task, TaskType, TaskID, State};

#[derive(Debug, Clone)]
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

    pub fn reset_timer(&mut self) {
        self.started = Instant::now();
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


    /// Get the first available Idle task to give to a worker
    // NOTE: Think of a better name for this. 
    pub fn get_idle_task(&mut self, id: i8, task_type: Option<TaskType>) -> Option<(Vec<String>, TaskType)> {
        let taskman_ref: Arc<Mutex<Vec<TimedTask>>> = Arc::clone(&self.list);
        let mut taskman: MutexGuard<'_, Vec<TimedTask>> = taskman_ref.lock().unwrap();
        // TODO: double-check the logic
        for timed_task in &mut (*taskman) {
            if timed_task.task.get_state() == State::Idle && (task_type == Some(timed_task.task.get_task_type()) || task_type == None){ // NOTE: double check logic of second logical statement
                timed_task.task.set_worker_id(id);
                timed_task.task.set_state(State::InProgress);
                timed_task.reset_timer();
                return Some((timed_task.task.get_path(), timed_task.task.get_task_type()))
            }         
        }
        None
    }

    /// update_state: Update the state of a task if it exists (return true)
    pub fn update_state(&mut self, task: String, state: State) -> Option<TaskType> {
        let taskman_ref: Arc<Mutex<Vec<TimedTask>>> = Arc::clone(&self.list);
        let mut taskman: MutexGuard<'_, Vec<TimedTask>> = taskman_ref.lock().unwrap();
        for timed_task in &mut (*taskman) {
            if timed_task.task.get_path().contains(&task) {
                timed_task.task.set_state(state);
                return Some(timed_task.task.get_task_type())
            }         
        }
        None
    }

    pub fn check_progress(&mut self, task: String, duration: Duration) {
        let taskman_ref: Arc<Mutex<Vec<TimedTask>>> = Arc::clone(&self.list);
        let mut taskman: MutexGuard<'_, Vec<TimedTask>> = taskman_ref.lock().unwrap();
        for timed_task in &mut (*taskman) {
            if timed_task.task.get_path().contains(&task) {
                timed_task.check_progress(duration);
            }         
        }
    }

    pub fn get_task_id(&mut self, path: String) -> Option<i8> {
        let taskman_ref: Arc<Mutex<Vec<TimedTask>>> = Arc::clone(&self.list);
        let mut taskman: MutexGuard<'_, Vec<TimedTask>> = taskman_ref.lock().unwrap();
        for timed_task in &mut (*taskman) {
            if timed_task.task.get_path().contains(&path) {
                return timed_task.task.get_task_id()
            }         
        }
        None
    }
    // Implement for debugging
    pub fn get_task(&mut self, path: String) -> Option<Task> {
        // TODO: double-check the logic
        for timed_task in  (*self.list.lock().unwrap()).clone() {
            if timed_task.task.get_path().contains(&path) {
                return Some(timed_task.task.clone())
            }         
        }
        None
    }

    pub fn task_completed(&mut self, path: String, reduce_type: ReduceType, nreduce: usize, nmap: usize, worker_id: i8) -> Result<(), TaskManagerError> {
        match self.update_state(path.clone(), State::Completed) {
            Some(TaskType::Map) => {
                // Remove completed tasks
                self.clean();
                match reduce_type {
                    ReduceType::Expedited => {
                        // add reduce task immediately
                        // TODO: double-check implementation logic
                        for i in 0..nreduce {
                            // TODO: get id from task
                            let mut task = Task::new(
                                vec!(format!("mr-{}-{}", self.get_task_id(path.clone()).unwrap(), i)), 
                                TaskType::Reduce,
                                TaskID::ReduceID,
                                );
                            task.set_worker_id(worker_id);
                            self.add_task(
                                task
                            );
                        }
                    }
                    ReduceType::Traditional => {
                        // add reduce task only if no map tasks remain (this only happens when the
                        // last map task is completed
                        if self.get_size(Some(TaskType::Map)) == 0 {
                            (0..nreduce)
                                .into_iter()
                                .for_each(|j| {
                                self.add_task(
                                    Task::new(
                                        (0..nmap)
                                            .into_iter()
                                            .map(|i| format!("mr-{}-{}", i, j))
                                            .collect::<Vec<String>>(),
                                        TaskType::Reduce,
                                        TaskID::ReduceID
                                    )
                                );
                            });
                        }
                    }
                }
                Ok(())
            }
            Some(TaskType::Reduce) => {
                // Remove completed tasks
                self.clean();
                Ok(())
            }
            None => {
                panic!("Task did not exist in the task manager.")
            }
        }
    }


    // pub fn check_task_type(&self) -> bool { } 

    /// clean: Remove completed tasks 
    fn clean(&mut self) {
        let taskman_ref: Arc<Mutex<Vec<TimedTask>>> = Arc::clone(&self.list);
        let mut taskman: MutexGuard<'_, Vec<TimedTask>> = taskman_ref.lock().unwrap();
        (*taskman).retain(|timed_task| timed_task.task.get_state() != State::Completed);
    }

    /// get_size: Get the size of the list of tasks remaining 
    pub fn get_size(&self, task_type: Option<TaskType>) -> usize {
        let taskman_ref: Arc<Mutex<Vec<TimedTask>>> = Arc::clone(&self.list);
        let taskman: MutexGuard<'_, Vec<TimedTask>> = taskman_ref.lock().unwrap();

        if let Some(task_type) = task_type {
            let mut count: usize = 0;
            (*taskman).iter().for_each(|task| {
                if task.task.get_task_type() == task_type {
                    count += 1
                };
            });
            count
        } else {
            (*taskman).len()
        }
    }
}


#[derive(Debug)]
pub enum TaskManagerError {
    TaskCompletedError,
}

impl std::error::Error for TaskManagerError {}

impl fmt::Display for TaskManagerError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      TaskManagerError::TaskCompletedError => write!(f, "Task Completed Error"),
    }
  }
}

#[cfg(test)]
mod test { }
