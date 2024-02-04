use alloc::sync::Arc;

use crate::fs::{open_file, OpenFlags};
use crate::mm::{translate_to_str, translated_refmut};
use crate::timer::get_time_ms;
use crate::task::{current_task, add_task, exit_current_and_run_next, suspend_and_run_next, current_user_token};

// task exit and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

// giving up CPU, always return 0
pub fn sys_yield() -> isize{
    trace!("kernel #0", "sys_yield is called");
    suspend_and_run_next();
    0
}

// get time in milliseconds
pub fn sys_get_time() -> isize{
    get_time_ms() as isize
}

// get pid
pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, so it cna return 0 in child process
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // return code store in x10
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translate_to_str(token, path);
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let task = current_task().unwrap();
        task.exec(all_data.as_slice());
        0
    } else {
        -1
    }
}

// return -1, if there's no child process has id equal to pid
// return -2, if the child process is still running
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_task().unwrap();

    // access current TaskControlBlock
    let mut inner = task.inner_exclusive_access();
    // find child process
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // release current TaskControlBlock
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)|
        // temporarily access child TaskControlBlock
        { p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid()) }
        // release child TaskControlBlock
    );
    if let Some((idx, _)) = pair {
        // remove the process
        let child = inner.children.remove(idx);
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // temporarily access child TaskControlBlock
        let exit_code = child.inner_exclusive_access().exit_code;
        // release child TaskControlBlock
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
}

#[allow(deprecated)]
#[allow(unreachable_code)]
pub fn sys_shutdown(failure: usize) -> ! {
    if failure == 0 {
        crate::sbi::shutdown(true);
    } else {
        crate::sbi::shutdown(false);
    }
    unreachable!();
}