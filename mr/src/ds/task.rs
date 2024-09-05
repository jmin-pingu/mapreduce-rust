#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum TaskType {
    Map,
    Reduce,
}

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Idle,
    InProgress,
    Completed,
}

/// A Task contains information about the tasks that a Worker will be assigned to 
/// complete by the Coordinator
// NOTE: we can also pair a path with a UUID so that tasks with the same path are distinguishable.
// However, this is pointless as paths should be unique (assumption)
#[derive(Debug, Clone)]
pub struct Task { 
    path: Vec<String>, 
    worker: Option<i8>,
    state: State,
    task_type: TaskType, 
}

impl Task {
    pub fn new(path: Vec<String>, state: State, task_type: TaskType) -> Self { 
        Task {
            path, 
            worker: None, 
            state,
            task_type,
        }
    }

    pub fn set_state(&mut self, state: State) { 
        self.state = state;
    }

    pub fn get_state(&self) -> State { 
        self.state.clone()
    }

    pub fn get_path(&self) -> Vec<String> { 
        self.path.clone()
    }

    pub fn get_task_type(&self) -> TaskType { 
        self.task_type.clone()
    }

    pub fn set_worker(&mut self, worker: i8) { 
        self.worker = Some(worker);
    }
}

