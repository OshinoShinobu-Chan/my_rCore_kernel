// module about task manager, including starting and switching tasks

mod context;
mod switch;

#[allow(clippy::module_inception)]
mod task;

use crate::loader::{get_num_app, get_app_data};
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use alloc::vec::Vec;
use lazy_static::*;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;

// Struct of task manager
// Function implemented on `TaskManager` deals with all task state transitions
// and task context switching
pub struct TaskManager {
    // total number of tasks
    num_app: usize,
    // use inner value to get mutable access
    inner: UPSafeCell<TaskManagerInner>,
}

// Inner of task manager
// Most of the data in task manager are here
pub struct TaskManagerInner {
    // task list
    tasks: Vec<TaskControlBlock>,
    // current task id
    current_task: usize,
}

lazy_static!{
    //initialize a global instance of task manager
    pub static ref TASK_MANAGER: TaskManager = {
        info!("kernel #0", "init TASK_MANAGER");
        let num_app = get_num_app();
        info!("kernel #0", "num_app = {}", num_app);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(get_app_data(i), i));
            info!("kernel #0", "Load app_{}", i);
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    // Run the first task in task list
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        // set the first task as `Running`
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        // set the first task as the next task for __switch
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("Unreachable in run_first_task");
    }

    // Change the status of current `Running` task to `Ready`
    // Note that there's at most one `Running` task
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    // Change the status of current `Running` task to `Exited`
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    // Find the next `Ready` task to run and return its id
    // In this case, the first `Ready` task is returned
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        // find the next task using iterator
        // map is used to turn the number in range to the real id of tasks
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    // Get the current `Running` task's token(satp)
    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_user_token()
    }

    // Get the current `Running` task's trap context
    fn get_current_trap_cx(&self) -> &'static mut TrapContext {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_trap_cx()
    }

    // Switch current `Running` task to the task we have found(chosen)
    // If there's no `Ready` task, exit the kernel with application completed
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            // change the state of the next task
            // note that the current task has already been marked as `Ready` or `Exited`
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            // switch context
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            trace!("kernel #0", "switch task from app_{}, ra:{:?} to app_{}, ra:{:?}", 
                    current, inner.tasks[current].task_cx, next, inner.tasks[next].task_cx);
            drop(inner);
            // drop the inner mannually, otherwise the __switch will change the inner
            // and cause panic for double mutable borrow
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            println!("All tasks completed, shutting down!");
            use crate::board::QEMUExit;
            crate::board::QEMU_EXIT_HANDLE.exit_success();
        }
    }
}

// suspend current task
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

// exit current task
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

// suspend current task and run next task
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

// exit current task and run next task
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

// Get the current `Running` task's token(satp)
pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

// Get the current `Running` task's trap context
pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}

// run first task
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

// run next task
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}