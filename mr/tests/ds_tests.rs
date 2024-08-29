use mr;
use mr::ds::{State, TaskType};

#[test]
fn add_task() {
    let result = mr::ds::Task::new("test.txt".to_string(), 1, State::Idle, TaskType::Map);
}

