use mr::ds::*;
use std::{
    time,
    thread
};
use mr::ds::task::*;
#[test]
fn test_intermediate() {
    let mut intermediate = intermediate::Intermediate::new();
    assert_eq!(intermediate.get("foo".to_string()), None);
    intermediate.insert("foo".to_string(), "bar0".to_string());
    assert_eq!(intermediate.get("foo".to_string()).unwrap()[0], "bar0".to_string());

    intermediate.insert("foo".to_string(), "bar1".to_string());
    intermediate.insert("foo".to_string(), "bar2".to_string());
    intermediate.insert("foo".to_string(), "bar3".to_string());
    assert_eq!(intermediate.get("foo".to_string()).unwrap()[1], "bar1".to_string());
    assert_eq!(intermediate.get("foo".to_string()).unwrap()[3], "bar3".to_string());
    assert_eq!(intermediate.get("foo".to_string()).unwrap().len(), 4);
}


#[test]
fn test_taskman_single_threaded() { 
    let mut taskman = taskman::TaskManager::new();
    let task1 = task::Task::new(vec!(String::from("a")), State::Idle, TaskType::Map);
    let task2 = task::Task::new(vec!(String::from("b"), String::from("c")), State::Idle, TaskType::Map);
    let task3 = task::Task::new(vec!(String::from("d"), String::from("f"), String::from("g")), State::Idle, TaskType::Map);
    let task4 = task::Task::new(vec!(String::from("h")), State::Idle, TaskType::Reduce);
    let task5 = task::Task::new(vec!(String::from("i")), State::Idle, TaskType::Map);
    let task6 = task::Task::new(vec!(String::from("j")), State::Idle, TaskType::Map);

    taskman.add_task(task1);
    taskman.update_state(String::from("a"), State::InProgress);
    let delay = time::Duration::from_millis(100);
    thread::sleep(delay);
    taskman.check_progress(String::from("a"), time::Duration::from_millis(500));
    assert_eq!(taskman.get_task("a".to_string()).unwrap().get_state(), State::InProgress);
    taskman.check_progress(String::from("a"), time::Duration::from_millis(100));
    assert_eq!(taskman.get_task("a".to_string()).unwrap().get_state(), State::Idle);

    // Size checks
    assert_eq!(taskman.get_size(None), 1);
    assert_eq!(taskman.get_size(Some(TaskType::Map)), 1);
    assert_eq!(taskman.get_size(Some(TaskType::Reduce)), 0);

    taskman.add_task(task2);
    taskman.add_task(task3);
    taskman.add_task(task4);

    // Size checks
    assert_eq!(taskman.get_size(None), 4);
    assert_eq!(taskman.get_size(Some(TaskType::Map)), 3);
    assert_eq!(taskman.get_size(Some(TaskType::Reduce)), 1);

    // Get the idle tasks
    assert_eq!(taskman.get_idle_task(0, None), Some((vec!(String::from("a")), TaskType::Map)));
    assert_eq!(taskman.get_idle_task(1, None), Some((vec!(String::from("b"), String::from("c")), TaskType::Map)));

    // Check state updated
    assert_eq!(taskman.get_task(String::from("a")).unwrap().get_state(), State::InProgress);
    assert_eq!(taskman.get_task(String::from("b")).unwrap().get_state(), State::InProgress);
    
    // pub fn get_idle_task(&mut self, id: i8, task_type: Option<TaskType>) -> Option<(Vec<String>, TaskType)> 

    // METHODS
    // pub fn add_task(&mut self, task: Task) 
    // pub fn get_idle_task(&mut self, id: i8, task_type: Option<TaskType>) -> Option<(Vec<String>, TaskType)> 
    // pub fn update_state(&mut self, task: String, state: State) -> Option<TaskType> 
    // pub fn check_progress(&mut self, task: String, duration: Duration) 
    // pub fn get_task(&mut self, path: String) -> Option<Task> 
    // pub fn task_completed(&mut self, task: String, reduce_type: ReduceType, nreduce: usize, nmap: usize, id: Option<i8>) -> Result<(), TaskManagerError> 
    // pub fn clean(&mut self) 
    // pub fn get_size(&self, task_type: Option<TaskType>) -> usize 
    // pub fn check_progress(&mut self, duration: Duration) 
}

#[test]
fn test_taskman_multi_threaded() { 

}

#[test]
fn test3() { }

#[test]
fn test4() { }

#[test]
fn test5() { }

#[test]
fn test6() { }
