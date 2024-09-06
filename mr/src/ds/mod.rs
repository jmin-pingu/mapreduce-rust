pub mod taskman;
pub mod task;
pub mod intermediate;

#[derive(PartialEq, Debug)]
pub enum MapReduceStatus {
    InProgress, 
    Completed,
}
