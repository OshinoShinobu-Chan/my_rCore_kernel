use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::fs::{open_file, OpenFlags};
use crate::mm::{translate_ref, translate_to_str, translated_refmut};
use crate::timer::get_time_ms;
use crate::task::{add_task, current_task, current_user_token, exit_current_and_run_next, pid2task, suspend_and_run_next, SignalAction, MAX_SIG};
use crate::task::SignalFlags;

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

pub fn sys_exec(path: *const u8, mut args: *const usize) -> isize {
    let token = current_user_token();
    let path = translate_to_str(token, path);
    let mut arg_vec: Vec<String> = Vec::new();
    loop {
        let arg_str_ptr = *translate_ref(token, args);
        if arg_str_ptr == 0 {
            break;
        }
        arg_vec.push(translate_to_str(token, arg_str_ptr as *const u8));
        unsafe {
            args = args.add(1);
        }
    }
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let task = current_task().unwrap();
        let argc = arg_vec.len();
        task.exec(all_data.as_slice(), arg_vec);
        argc as isize
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

pub fn sys_sigprocmask(mask: u32) -> isize {
    if let Some(task) = current_task() {
        let mut inner = task.inner_exclusive_access();
        let old_mask = inner.signal_mask;
        if let Some(flag) = SignalFlags::from_bits(mask) {
            inner.signal_mask = flag;
            old_mask.bits() as isize
        } else {
            -1
        }
    } else {
        -1
    }
}

/// Check if the sigaction is right, return true if the sigaction is illegal
fn check_sigaction_error(signal: SignalFlags, action: usize, old_action: usize) -> bool {
    if action == 0
        || old_action == 0
        || signal == SignalFlags::SIGKILL
        || signal == SignalFlags::SIGSTOP
    {
        true
    } else {
        false
    }
}

pub fn sys_sigaction(
    signum: i32,
    action: *const SignalAction,
    old_action: *mut SignalAction,
) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if signum as usize > MAX_SIG {
        return -1;
    }
    if let Some(flag) = SignalFlags::from_bits(1 << signum) {
        if check_sigaction_error(flag, action as usize, old_action as usize) {
            return -1;
        }
        let prev_action = inner.signal_actions.table[signum as usize];
        *translated_refmut(token, old_action) = prev_action;
        inner.signal_actions.table[signum as usize] = *translate_ref(token, action);
        0
    } else {
        -1
    }
}

pub fn sys_kill(pid: usize, signum: i32) -> isize {
    if let Some(task) = pid2task(pid) {
        if let Some(flag) = SignalFlags::from_bits(1 << signum) {
            // insert the signal if legal
            let mut task_ref = task.inner_exclusive_access();
            if task_ref.signals.contains(flag) {
                return -1;
            }
            task_ref.signals.insert(flag);
            0
        } else {
            -1
        }
    } else {
        -1
    }
}

pub fn sys_sigreturn() -> isize {
    if let Some(task) = current_task() {
        let mut inner = task.inner_exclusive_access();
        inner.handling_sig = -1;
        // restore trap context
        let trap_cx = inner.get_trap_cx();
        *trap_cx = inner.trap_ctx_backup.unwrap();
        trap_cx.x[10] as isize
    } else {
        -1
    }
}