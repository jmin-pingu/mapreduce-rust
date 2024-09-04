// How do I implement "waiting for all maps to be completed"
#[derive(Debug)]
pub enum ReduceType {
    Expedited,
    Traditional,
}

pub struct Worker {
    worker_id: i8,    
    reduce_type: ReduceType,
    functions: crate::ExternalFunctions,
    // NOTE: do I want to add additional metadata?
}

impl Worker {
    pub fn new(worker_id: i8, reduce_type: ReduceType, functions: crate::ExternalFunctions) -> Self {
        Worker {
            worker_id, 
            reduce_type,
            functions
        }
    }

    pub fn do_work() {

    }

    pub fn do_reduce() {

    }

    pub fn do_map() {

    }
}

