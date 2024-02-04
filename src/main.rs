//#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
extern crate bitflags;

use core::arch::global_asm;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
#[macro_use]
mod log;
mod config;
mod drivers;
pub mod fs;
mod lang_items;
pub mod mm;
mod sbi;
mod sync;
pub mod syscall;
pub mod task;
mod timer;
pub mod trap;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

// the entry of kernel
#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    println!("[kernel] Hello, world!");
    trace!("kernel #0", "Hello, world!");
    debug!("kernel #0", "Hello, world!");
    info!("kernel #0", "Hello, world!");
    warn!("kernel #0", "Hello, world!");
    error!("kernel #0", "Hello, world!");
    error!(
        "kernel #0",
        "Note: infomation above are just test for logging system!"
    );
    mm::init();
    info!("kernel #0", "memory space initialized");
    mm::remap_test();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    fs::list_apps();
    task::add_initproc();
    info!("kernel #0", "initproc added");
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe {
            (a as *mut u8).write_volatile(0)
        }
    });
}