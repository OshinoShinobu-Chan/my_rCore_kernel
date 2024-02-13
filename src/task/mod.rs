// module about task manager, including starting and switching tasks

use crate::{fs::{open_file, OpenFlags}, sbi::shutdown};

use self::{context::TaskContext, manager::remove_from_pid2task, task::{TaskStatus, TaskControlBlock}};

mod context;
mod switch;

#[allow(clippy::module_inception)]
mod task;
mod pid;
mod manager;
mod processor;
mod signal;
mod action;
use alloc::sync::Arc;
use lazy_static::lazy_static;
pub use processor::{
    current_task, current_trap_cx, current_user_token, run_tasks, scheduler, take_current_task,
    Processor
};
pub use manager::{ add_task, pid2task };
pub use signal::{ MAX_SIG, SignalFlags };
pub use action::{ SignalAction, SignalActions };

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

    //remove from pid2task
    remove_from_pid2task(task.getpid());

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
    // drop file descriptors
    inner.fd_table.clear();
    drop(inner);
    // release current TaskControlBlock
    drop(task);
    // no TaskContext neede to save, use an empty one
    let mut _unused = TaskContext::zero_init();
    scheduler(&mut _unused as *mut _);
}

lazy_static!{
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new({
        let inode = open_file("initproc", OpenFlags::RDONLY).unwrap();
        let v = inode.read_all();
        TaskControlBlock::new(v.as_slice())
});
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
}

pub fn check_signal_error_of_current() -> Option<(i32, &'static str)> {
    let task = current_task().unwrap();
    let task_inner = task.inner_exclusive_access();
    task_inner.signals.check_error()
}

pub fn current_add_signal(signal: SignalFlags) {
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    task_inner.signals |= signal;
}

fn call_kernel_signal_handler(signal: SignalFlags) {
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    match signal {
        SignalFlags::SIGSTOP => {
            task_inner.frozen = true;
            task_inner.signals ^= SignalFlags::SIGSTOP;
        }
        SignalFlags::SIGCONT => {
            if task_inner.signals.contains(SignalFlags::SIGCONT) {
                task_inner.signals ^= SignalFlags::SIGCONT;
                task_inner.frozen = false;
            }
        }
        _ => {
            task_inner.killed = true;
        }
    }
}

fn call_user_signal_handler(sig: usize, signal: SignalFlags) {
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();

    let handler = task_inner.signal_actions.table[sig].handler;
    if handler != 0 {
        // user handler exists
        task_inner.handling_sig = sig as isize;
        task_inner.signals ^= signal;

        //backup trap context
        let trap_cx = task_inner.get_trap_cx();
        task_inner.trap_ctx_backup = Some(*trap_cx);

        //modify trapcontext
        trap_cx.sepc = handler;

        // put args(a0)
        trap_cx.x[10] = sig;
    } else {
        info!("kernel #0", "task/call_user_signal_handler: default action: ignore it or kill process");
    }
}

fn check_pending_signals() {
    for sig in 0..(MAX_SIG + 1) {
        let task = current_task().unwrap();
        let task_inner = task.inner_exclusive_access();
        let signal = SignalFlags::from_bits(1 << sig).unwrap();
        // if the signal is received and is not masked
        if task_inner.signals.contains(signal) && (!task_inner.signal_mask.contains(signal)) {
            let mut masked = true;
            let handling_sig = task_inner.handling_sig;
            if handling_sig == -1 {
                masked = false;
            } else {
                let handling_sig = handling_sig as usize;
                if !task_inner.signal_actions.table[handling_sig]
                    .mask
                    .contains(signal)
                {
                    masked = false;
                }
            }
            if !masked {
                drop(task_inner);
                drop(task);
                let kernel_sig = SignalFlags::SIGKILL 
                    | SignalFlags::SIGSTOP 
                    | SignalFlags::SIGCONT
                    | SignalFlags::SIGDEF;
                if kernel_sig.contains(signal) {
                    // signal is a kernel signal
                    call_kernel_signal_handler(signal);
                } else {
                    // signal is a user signal
                    call_user_signal_handler(sig, signal);
                    return;
                }
            }
        }
    }
} 

pub fn handle_signals() {
    loop {
        check_pending_signals();
        let (frozen, killed) = {
            let task = current_task().unwrap();
            let task_inner = task.inner_exclusive_access();
            (task_inner.frozen, task_inner.killed)
        };
        if !frozen || killed {
            break;
        }
        suspend_and_run_next();
    }
}