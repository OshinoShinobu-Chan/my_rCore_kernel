use alloc::{collections::{BTreeMap, VecDeque}, sync::Arc};
use lazy_static::*;

use super::task::TaskControlBlock;
use crate::sync::UPSafeCell;

pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

// A simple FIFO scheduler
impl TaskManager {
    // Create an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    // Add a task to TaskManager
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    // Remove the first task and return it, return None if TaskManager is empty
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }
}

// Create a global instance of TaskManager
lazy_static!{
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> = 
        unsafe { UPSafeCell::new(TaskManager::new()) };
    pub static ref PID2TCB: UPSafeCell<BTreeMap<usize, Arc<TaskControlBlock>>> = 
        unsafe { UPSafeCell::new(BTreeMap::new()) };
}

// Wrapper of add
pub fn add_task(task: Arc<TaskControlBlock>) {
    PID2TCB
        .exclusive_access()
        .insert(task.getpid(), Arc::clone(&task));
    TASK_MANAGER.exclusive_access().add(task);
}

// Wrapper of fetch
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}

pub fn pid2task(pid: usize) -> Option<Arc<TaskControlBlock>> {
    let map = PID2TCB.exclusive_access();
    map.get(&pid).map(Arc::clone)
}

pub fn remove_from_pid2task(pid: usize) {
    let mut map = PID2TCB.exclusive_access();
    if map.remove(&pid).is_none() {
        panic!("Task with pid {} not found", pid);
    }
}