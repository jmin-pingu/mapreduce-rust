use mr::ds::*;
use mr::worker::ReduceType;
use std::{
    time,
    thread,
    sync::{Arc, Mutex}
};
use tokio::task;
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
    let task1 = Task::new(vec!(String::from("a")), TaskType::Map, TaskID::MapID(0));
    let task2 = Task::new(vec!(String::from("b"), String::from("c")), TaskType::Map, TaskID::MapID(1));
    let task3 = Task::new(vec!(String::from("d"), String::from("f"), String::from("g")), TaskType::Map, TaskID::MapID(2));
    let task4 = Task::new(vec!(String::from("h")), TaskType::Map, TaskID::MapID(3));
    let task5 = Task::new(vec!(String::from("i")), TaskType::Reduce, TaskID::ReduceID);
    let task6 = Task::new(vec!(String::from("j")), TaskType::Reduce, TaskID::ReduceID);

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
    taskman.add_task(task5);
    taskman.add_task(task6);

    // Size checks
    assert_eq!(taskman.get_size(None), 6);
    assert_eq!(taskman.get_size(Some(TaskType::Map)), 4);
    assert_eq!(taskman.get_size(Some(TaskType::Reduce)), 2);

    // Get the idle tasks
    assert_eq!(taskman.get_idle_task(0, None), Some((vec!(String::from("a")), TaskType::Map)));
    assert_eq!(taskman.get_idle_task(1, None), Some((vec!(String::from("b"), String::from("c")), TaskType::Map)));

    // Check state updated
    assert_eq!(taskman.get_task(String::from("a")).unwrap().get_state(), State::InProgress);
    assert_eq!(taskman.get_task(String::from("b")).unwrap().get_state(), State::InProgress);
    
    taskman.task_completed(String::from("a"), ReduceType::Traditional, 2, 2, 0).unwrap();
    assert_eq!(taskman.get_size(None), 5);
    assert_eq!(taskman.get_size(Some(TaskType::Map)), 3);
    assert_eq!(taskman.get_size(Some(TaskType::Reduce)), 2);


    assert_eq!(taskman.get_idle_task(2, Some(TaskType::Reduce)), Some((vec!(String::from("i")), TaskType::Reduce)));
    assert_eq!(taskman.get_idle_task(3, Some(TaskType::Reduce)), Some((vec!(String::from("j")), TaskType::Reduce)));

    taskman.task_completed(String::from("b"), ReduceType::Traditional, 2, 2, 0).unwrap();
    assert_eq!(taskman.get_size(None), 4);
    assert_eq!(taskman.get_size(Some(TaskType::Map)), 2);
    assert_eq!(taskman.get_size(Some(TaskType::Reduce)), 2);

    // Complete ReduceType::Expedited, so we should have mr-100-0 and mr-100-1 of TaskType::Reduce
    // added to taskman
    taskman.task_completed(String::from("d"), ReduceType::Expedited, 2, 2, 100).unwrap();
    assert_eq!(taskman.get_size(None), 5);
    assert_eq!(taskman.get_size(Some(TaskType::Map)), 1);
    assert_eq!(taskman.get_size(Some(TaskType::Reduce)), 4);

    assert_eq!(taskman.get_task(String::from("mr-100-0")).unwrap().get_worker_id().unwrap(), 100);
    assert_eq!(taskman.get_task(String::from("mr-100-1")).unwrap().get_worker_id().unwrap(), 100);

    // Get rid of the pre-existing reduce tasks
    taskman.task_completed(String::from("i"), ReduceType::Traditional, 2, 2, 0).unwrap();
    taskman.task_completed(String::from("j"), ReduceType::Traditional, 2, 2, 0).unwrap();
    assert_eq!(taskman.get_size(None), 3);
    assert_eq!(taskman.get_size(Some(TaskType::Map)), 1);
    assert_eq!(taskman.get_size(Some(TaskType::Reduce)), 2);

    // Get rid of the added reduce tasks when we completed ReduceType::Expedited
    taskman.task_completed(String::from("mr-100-0"), ReduceType::Traditional, 2, 2, 0).unwrap();
    taskman.task_completed(String::from("mr-100-1"), ReduceType::Traditional, 2, 2, 0).unwrap();
    assert_eq!(taskman.get_size(None), 1);
    assert_eq!(taskman.get_size(Some(TaskType::Map)), 1);
    assert_eq!(taskman.get_size(Some(TaskType::Reduce)), 0);

    // Finally, double-check traditional. There should be nreduce new reduce tasks since there are
    // no map tasks remaining
    taskman.task_completed(String::from("h"), ReduceType::Traditional, 2, 5, 0).unwrap();
    assert_eq!(taskman.get_size(None), 2);
    assert_eq!(taskman.get_size(Some(TaskType::Map)), 0);
    assert_eq!(taskman.get_size(Some(TaskType::Reduce)), 2);

    // Double check the length of the lists. The different permutations of "mr-X-0" where X is a
    // map task id is just to also check the functionality of get_task
    assert_eq!(taskman.get_task(String::from("mr-0-0")).unwrap().get_path().len(), 5);
    assert_eq!(taskman.get_task(String::from("mr-1-1")).unwrap().get_path().len(), 5);
    assert_eq!(taskman.get_task(String::from("mr-2-0")).unwrap().get_path().len(), 5);
    assert_eq!(taskman.get_task(String::from("mr-3-1")).unwrap().get_path().len(), 5);
    assert_eq!(taskman.get_task(String::from("mr-4-0")).unwrap().get_path().len(), 5);

    // Complete the remaining reduce tasks
    taskman.task_completed(String::from("mr-0-0"), ReduceType::Traditional, 2, 5, 0).unwrap();
    taskman.task_completed(String::from("mr-0-1"), ReduceType::Traditional, 2, 5, 0).unwrap();
    assert_eq!(taskman.get_size(None), 0);
    assert_eq!(taskman.get_size(Some(TaskType::Map)), 0);
    assert_eq!(taskman.get_size(Some(TaskType::Reduce)), 0);
    // METHODS
    // fn add_task(&mut self, task: Task) 
    // fn get_task(&mut self, path: String) -> Option<Task> 
    // fn get_idle_task(&mut self, id: i8, task_type: Option<TaskType>) -> Option<(Vec<String>, TaskType)> 
    // fn task_completed(&mut self, task: String, reduce_type: ReduceType, nreduce: usize, nmap: usize, id: Option<i8>) -> Result<(), TaskManagerError> 
    // fn clean(&mut self) 
    // fn get_size(&self, task_type: Option<TaskType>) -> usize 
    // fn check_progress(&mut self, duration: Duration) 
}

#[test]
fn test_taskman_multi_threaded() { 
    let mut taskman = taskman::TaskManager::new();
    // Add several tasks
    (0..100).for_each(|i| taskman.add_task(Task::new(vec!(i.to_string()), TaskType::Map, TaskID::MapID(i))));
    assert_eq!(taskman.get_size(None), 100);
    assert_eq!(taskman.get_size(Some(TaskType::Map)), 100);
    assert_eq!(taskman.get_size(Some(TaskType::Reduce)), 0);

    let ts_taskman = Arc::new(Mutex::new(taskman));
    let mut handles = vec![];
   

    // Get rid of n tasks
    let n: i8 = 50;
    for k in 0..n {
        let task_ref= Arc::clone(&ts_taskman);
        let handle = thread::spawn(move ||
            {
                let mut thread_taskman = task_ref.lock().unwrap();
                let (paths, task_type) = (*thread_taskman).get_idle_task(k, None).unwrap();
                assert_eq!(task_type, TaskType::Map);
                let path = paths[0].clone();
                assert!(path.parse::<usize>().expect("Failed to parse path") <= n as usize);
                thread_taskman.task_completed(path, ReduceType::Traditional, 2, 2, 0).unwrap();

            });
        handles.push(handle);
    };

    for handle in handles {
        handle.join().unwrap();
    }

    let task_ref= Arc::clone(&ts_taskman);
    {
        let safe_taskman = task_ref.lock().unwrap();
        assert_eq!(safe_taskman.get_size(None), 100 - n as usize);
        assert_eq!(safe_taskman.get_size(Some(TaskType::Map)), 100 - n as usize);
        assert_eq!(safe_taskman.get_size(Some(TaskType::Reduce)), 0);
    }

    // Get rid of the rest of tasks
    let mut handles = vec![];
    for k in 0..(100 - n) {
        let task_ref= Arc::clone(&ts_taskman);
        let handle = thread::spawn(move ||
            {
                let mut thread_taskman = task_ref.lock().unwrap();
                let (paths, task_type) = (*thread_taskman).get_idle_task(k, None).unwrap();
                assert_eq!(task_type, TaskType::Map);
                let path = paths[0].clone();
                assert!(path.parse::<usize>().expect("Failed to parse path") <= 100 as usize && path.parse::<usize>().expect("Failed to parse path") >= n as usize);
                thread_taskman.task_completed(path, ReduceType::Traditional, 10, 2, 0).unwrap();

            });
        handles.push(handle);
    };

    for handle in handles {
        handle.join().unwrap();
    }

    {
        let task_ref= Arc::clone(&ts_taskman);
        let safe_taskman = task_ref.lock().unwrap();
        assert_eq!(safe_taskman.get_size(None), 10 as usize);
        assert_eq!(safe_taskman.get_size(Some(TaskType::Map)), 0);
        assert_eq!(safe_taskman.get_size(Some(TaskType::Reduce)), 10);
    }


    let mut handles = vec![];
    for k in 0..10 {
        let task_ref= Arc::clone(&ts_taskman);
        let handle = thread::spawn(move ||
            {
                let mut thread_taskman = task_ref.lock().unwrap();
                let (paths, task_type) = (*thread_taskman).get_idle_task(k, None).unwrap();
                assert_eq!(task_type, TaskType::Reduce);
                let path = paths[0].clone();
                thread_taskman.task_completed(path, ReduceType::Traditional, 10, 2, 0).unwrap();

            });
        handles.push(handle);
    };

    for handle in handles {
        handle.join().unwrap();
    }
}

#[tokio::test]
async fn async_taskman_multithread_completion() { 
    let mut taskman = taskman::TaskManager::new();
    // Add several tasks
    (0..100).for_each(|i| taskman.add_task(Task::new(vec!(i.to_string()), TaskType::Map, TaskID::MapID(i))));
    assert_eq!(taskman.get_size(None), 100);
    assert_eq!(taskman.get_size(Some(TaskType::Map)), 100);
    assert_eq!(taskman.get_size(Some(TaskType::Reduce)), 0);

    let ts_taskman = Arc::new(Mutex::new(taskman));
    let mut completed: bool = false;

    while !completed {
        // Update completed when taskman is empty
        
        let task_ref= Arc::clone(&ts_taskman);
        let join = task::spawn(async move { 
            {
                let mut thread_taskman = task_ref.lock().unwrap();
                let (paths, _) = (*thread_taskman).get_idle_task(1, None).expect("No tasks remaining");
                let path = paths[0].clone();
                println!("Completed {}", path);
                thread_taskman.task_completed(path, ReduceType::Traditional, 10, 2, 0).unwrap();
            }});
        join.await.unwrap();
        {
            let task_ref= Arc::clone(&ts_taskman);
            let safe_taskman = task_ref.lock().unwrap();
            if safe_taskman.get_size(None) == 0 {
                completed = true;
            }
        }
    };
   
}

#[tokio::test]
async fn async_taskman_multithread_delays_completion() { 
    let mut taskman = taskman::TaskManager::new();
    // Add several tasks
    (0..100).for_each(|i| taskman.add_task(Task::new(vec!(i.to_string()), TaskType::Map, TaskID::MapID(i))));
    assert_eq!(taskman.get_size(None), 100);
    assert_eq!(taskman.get_size(Some(TaskType::Map)), 100);
    assert_eq!(taskman.get_size(Some(TaskType::Reduce)), 0);

    let ts_taskman = Arc::new(Mutex::new(taskman));
    let mut completed: bool = false;

    while !completed {
        // Update completed when taskman is empty
        
        let task_ref= Arc::clone(&ts_taskman);
        let join = task::spawn(async move { 
            {
                let mut thread_taskman = task_ref.lock().unwrap();
                let (paths, _) = (*thread_taskman).get_idle_task(1, None).expect("No tasks remaining");
                let path = paths[0].clone();
                println!("Started {}", path);
                thread_taskman.task_completed(path, ReduceType::Traditional, 10, 2, 0).unwrap();
            }});
        join.await.unwrap();
        {
            let task_ref= Arc::clone(&ts_taskman);
            let safe_taskman = task_ref.lock().unwrap();
            if safe_taskman.get_size(None) == 0 {
                completed = true;
            }
        }
    };
}

#[tokio::test]
async fn async_taskman_xxx() { }
