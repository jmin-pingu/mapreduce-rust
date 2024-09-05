use crate::ds::task::TaskType;
use crate::worker::ReduceType;

#[tarpc::service]
pub trait TaskService {
    /// get_task: an RPC to get a task of type TaskType from the Coordinator's TaskManager. Returns
    /// the path of the Task and whether all tasks are completed
    ///
    /// # Arguments
    ///
    /// * `id`: the worker id
    /// * `task_type`: the desired TaskType. If None, then gets either TaskType::Map or TaskType::Reduce, with a priority for TaskType::Map.
    ///
    async fn get_task(id: i8, task_type: Option<TaskType>) -> Option<(Vec<String>, TaskType)>;

    /// completed_task: a RPC which communicates whether a task is completed, where the task is
    /// denoted by its path. The function returns a bool to indicate whether communication was
    /// successful
    ///
    /// # Arguments
    ///
    /// * `task`: the path of the task that has been completed
    ///
    async fn completed_task(task: String, reduce_type: ReduceType, nreduce: usize, nmap: usize, id: Option<i8>);

    /// echo: an example RPC which has the same functionality as the `echo` syscall to test tarpc
    /// functionality
    ///
    /// # Arguments
    ///
    /// * `input`: the string that will be echoed back
    async fn echo(input: String) -> String;
}
