use crate::ds::TaskType;

#[tarpc::service]
pub trait TaskService {
    /// DEFINITION
    ///
    /// # Arguments
    ///
    /// * `var`
    async fn get_task(id: i8, task_type: Option<TaskType>) -> Option<String>;

    /// DEFINITION
    ///
    /// # Arguments
    ///
    /// * `var`
    async fn completed_task(task: String) -> bool;

    /// An example RPC to test functionality of tarpc
    ///
    /// # Arguments
    ///
    /// * `input`: input that will be echoed back
    async fn echo(input: String) -> String;
}
