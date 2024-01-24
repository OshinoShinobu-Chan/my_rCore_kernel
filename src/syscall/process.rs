use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};

// task exit and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    info!("kernel #0", "Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

// giving up CPU, always return 0
pub fn sys_yield() -> isize{
    trace!("kernel #0", "sys_yield is called");
    suspend_current_and_run_next();
    0
}