#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::arch::global_asm;

mod sbi;
#[macro_use]
mod console;
#[macro_use]
mod log;
mod lang_items;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    panic!("Deliberate panic");
    unreachable!();
    sbi::shutdown(true);
    
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