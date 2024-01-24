//#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::arch::global_asm;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
#[macro_use]
mod log;
mod config;
mod lang_items;
pub mod loader;
mod sbi;
mod sync;
pub mod syscall;
mod task;
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
    trap::init();
    loader::load_app();
    task::run_first_task();
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