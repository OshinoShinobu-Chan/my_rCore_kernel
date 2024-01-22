//#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::arch::global_asm;

#[macro_use]
mod console;
#[macro_use]
mod log;
pub mod batch;
mod lang_items;
mod sbi;
mod sync;
pub mod syscall;
pub mod trap;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

// the entry of kernel
#[no_mangle]
pub fn rust_main() -> ! {
    extern "C" {
        fn stext(); // begin addr of text segment
        fn etext(); // end addr of text segment
        fn srodata(); // begin addr of rodata segment
        fn erodata(); // end addr of rodata segment
        fn sdata(); // begin addr of data segment
        fn edata(); // end addr of data segment
        fn sbss(); // begin addr of bss segment
        fn ebss(); // end addr of bss segment
        fn boot_stack_lower_bound(); // stack lower bound
        fn boot_stack_top(); // stack top
    }
    clear_bss();
    println!("[kernel] Hello, world!");
    trace!(
        "kernel #0",
        ".text [{:#x}, {:#x})",
        stext as usize,
        etext as usize
    );
    debug!(
        "kernel #0",
        ".rodata [{:#x}, {:#x})",
        srodata as usize,
        erodata as usize
    );
    info!{
        "kernel #0",
        ".data [{:#x}, {:#x})",
        sdata as usize,
        edata as usize
    };
    warn!(
        "kernel #0",
        "boot_stack top=bottom={:#x}, lower_bound={:#x}",
        boot_stack_top as usize,
        boot_stack_lower_bound as usize
    );
    error!(
        "kernel #0",
        ".bss [{:#x}, {:#x})",
        sbss as usize,
        ebss as usize
    );
    error!(
        "kernel #0",
        "Note: infomation above are just test for logging system!"
    );
    trap::init();
    batch::init();
    batch::run_next_app();
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