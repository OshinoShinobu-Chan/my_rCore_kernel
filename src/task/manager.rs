use alloc::{collections::VecDeque, sync::Arc};
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
}

// Wrapper of add
pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access().add(task);
}

// Wrapper of fetch
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}