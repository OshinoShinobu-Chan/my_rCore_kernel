mod context;

use crate::task::current_user_token;
use crate::{syscall::syscall, task::current_trap_cx};
use crate::config::{TRAMPOLINE, TRAP_CONTEXT};
#[allow(unused)]
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::set_next_trigger;
use core::arch::{global_asm, asm};
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};

global_asm!(include_str!("trap.S"));

// initialize CSR `stvec` as the entry of `__alltraps`
pub fn init() {
    set_kernel_trap_entry();
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

// timer interrupt enabled
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[no_mangle]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let cx = current_trap_cx();
    let scause = scause::read(); // get trap cause
    let stval = stval::read(); // get extra value
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            trace!("kernel #0", "Kernel get a syscall, id = {}", cx.x[17]);
            cx.sepc += 4; // return to the next instruction
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            warn!("kernel #0", "PageFault in application, kernel killed it.");
            cx.sepc += 4; // prepared to call sys_exit
            cx.x[10] = syscall(93, [100, 0, 0]) as usize;
            panic!("");
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            warn!("kernel #0", "IllegalInstruction in application, bad instruction:{:#x} \
            kernel killed it.", cx.sepc);
            cx.sepc += 4; // prepared to call sys_exit
            cx.x[10] = syscall(93, [101, 0, 0]) as usize;
            panic!("");
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            trace!("kernel #0", "SupervisorTimer Interrupt");
            // set next timer
            set_next_trigger();
            // change to another task
            suspend_current_and_run_next()
        }
        _ => {
            warn!("kernel #0", "Unexpected exception: {:?}, stval = {:#x}, kernel killed it.", 
                    scause.cause(), stval);
            cx.sepc += 4; // prepared to call sys_exit
            cx.x[10] = syscall(93, [102, 0, 0]) as usize;
            panic!(
                "Unexpected exception {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    trap_return();
}

#[no_mangle]
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    //println!("...");
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,
            in("a1") user_satp,
            options(noreturn)
        );
    }
}

#[no_mangle]
pub fn trap_from_kernel() -> ! {
    //panic!("A trap from kernel!");
    trap_return();
}

pub use context::TrapContext;