#[derive(Clone)]
pub struct Coordinator {
    pub taskman: crate::ds::TaskManager, // NOTE: what if we have a daemon looping through the
                                          // task_manager looking for idle map jobs?
}

use tarpc::context;

// This is the service definition. It looks a lot like a trait definition.
#[tarpc::service]
pub trait Worker {
    /// DEFINITION
    ///
    /// # Arguments
    ///
    /// * `var`
    async fn get_task(id: i8) -> Option<String>;

    /// DEFINITION
    ///
    /// # Arguments
    ///
    /// * `var`
    async fn completed_task(task: String) -> bool;

    /// DEFINITION
    ///
    /// # Arguments
    ///
    /// * `var`
    async fn test(input: String) -> String;
}

impl Worker for Coordinator {
    async fn get_task(mut self, _: context::Context, id: i8) -> Option<String> {
        // crate::ds::Task::new("TimedTask".to_string(), 1, crate::ds::State::Idle, crate::ds::TaskType::Map)
        self.taskman.get_task(id)     
    }

    async fn completed_task(self, _: context::Context, task: String) -> bool { 
        // Change self.task_manager
        true 
    }    

    async fn test(self, _: context::Context, input: String) -> String {
        format!("Testing {input}!")
    }
}
