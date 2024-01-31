
use alloc::sync::Arc;
use lazy_static::lazy_static;

use crate::{sync::UPSafeCell, trap::TrapContext};

use super::{task::{TaskControlBlock, TaskStatus}, context::TaskContext, manager::fetch_task, switch::__switch};


// Processor managment structure
pub struct Processor {
    // The task curretly executing on the current processor
    current: Option<Arc<TaskControlBlock>>,
    // The basic control flow of each core, helping to select and switch process
    idle_task_cx: TaskContext,
}

impl Processor {
    // Create an empty Processor
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }
    // Get mutable reference to `idle_task_cx`
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }
    // Get current task and move its ownership
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }
    // Get current task and clone it
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
}

// Create a global instance of Processor
lazy_static!{
    pub static ref PROCESSOR: UPSafeCell<Processor> = 
        unsafe { UPSafeCell::new(Processor::new()) };
}

// Wrapper of take_current
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

// Wrapper of current
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

// Get the token of current task
pub fn current_user_token() -> usize {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_user_token()
}

// Get the trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx() as &mut TrapContext
}

// Loop `fetch_task` to get the process that needs to run, and switch the process
// through `__switch`
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // get next task's TaskControlBlock
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            drop(task_inner);
            processor.current = Some(task);
            drop(processor);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        }
    }
}

// Return to idle control flow for new scheduling
pub fn scheduler(switch_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switch_task_cx_ptr, idle_task_cx_ptr);
    }
}