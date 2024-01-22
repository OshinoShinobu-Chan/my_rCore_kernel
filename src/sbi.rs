//use core::arch::asm;

pub fn console_putchar(c: usize) {
    // let ret: usize;
    // let arg0: usize = c;
    // let arg1: usize = 0;
    // let arg2: usize = 0;
    // let arg3: usize = 0;
    // let which: usize = 1;
    // unsafe {
    //     asm!("ecall",
    //         in("a0") arg0, in("a1") arg1, in("a2") arg2, in("a3") arg3, in("a7") which,
    //         lateout("a0") ret,
    //     );
    // }
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c);
}

pub fn shutdown(failure: bool) -> ! {
    use sbi_rt::{system_reset, NoReason, Shutdown, SystemFailure};
    if !failure {
        system_reset(Shutdown, NoReason);
    } else {
        system_reset(Shutdown, SystemFailure);
    }
    unreachable!()
}
