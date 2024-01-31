// module about task manager, including starting and switching tasks

use crate::{loader::get_app_data_by_name, sbi::shutdown};

use self::{context::TaskContext, task::{TaskStatus, TaskControlBlock}};

mod context;
mod switch;

#[allow(clippy::module_inception)]
mod task;
mod pid;
mod manager;
mod processor;
use alloc::sync::Arc;
use lazy_static::lazy_static;
pub use processor::{
    current_task, current_trap_cx, current_user_token, run_tasks, scheduler, take_current_task,
    Processor
};
pub use manager::add_task;

pub const IDLE_PID: usize = 0;

pub fn suspend_and_run_next() {
    let task = take_current_task().unwrap();

    // get current TaskControlBlock
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // drop current TaskControlBlock

    add_task(task);
    // goto scheduling
    scheduler(task_cx_ptr);
}

pub fn exit_current_and_run_next(exit_code: i32) {
    let task = take_current_task().unwrap();

    let pid = task.getpid();
    if pid == IDLE_PID {
        info!("kernel #0", "Idle task exited with exit_code {}, shutdown ...", exit_code);
        if exit_code != 0{
            shutdown(true)
        } else {
            shutdown(false)
        }
    }

    // access current TaskControlBlock
    let mut inner = task.inner_exclusive_access();
    inner.task_status = TaskStatus::Zombie;
    inner.exit_code = exit_code;

    // access initproc TaskControlBlock
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // release initproc TaskControlBlock

    inner.children.clear();
    // dealloc user space
    inner.memory_set.recycle_data_pages();
    drop(inner);
    // release current TaskControlBlock
    drop(task);
    // no TaskContext neede to save, use an empty one
    let mut _unused = TaskContext::zero_init();
    scheduler(&mut _unused as *mut _);
}

lazy_static!{
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(TaskControlBlock::new(
        get_app_data_by_name("initproc").unwrap()
    ));
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
}