#[allow(deprecated)]
use sbi_rt::legacy::console_getchar;

use crate::task::{exit_current_and_run_next, suspend_and_run_next};

use super::File;

/// Standard input
pub struct Stdin;
/// Standard output
pub struct Stdout;
#[allow(unused)]
/// Standard error
pub type Stderr = Stdout;

impl File for Stdin {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        false
    }
    #[allow(deprecated)]
    fn read(&self, mut user_buf: crate::mm::UserBuffer) -> usize {
        if user_buf.len() == 0 {
            return 0;
        }
        if user_buf.len() != 1 {
            warn!("kernel #0", "Stdin: only support read size = 1, kill this call");
            exit_current_and_run_next(-9);
        }
        let mut c: usize;
        loop {
            c = console_getchar();
            if c == 0 {
                suspend_and_run_next();
                continue;
            } else {
                break;
            }
        }
        let ch = c as u8;
        unsafe {
            user_buf.buffers[0].as_mut_ptr().write_volatile(ch);
        }
        1
    }
    fn write(&self, _buf: crate::mm::UserBuffer) -> usize {
        warn!("kernel #0", "Stdin: not writable, kill this call");
        exit_current_and_run_next(-9);
        0
    }
}

impl File for Stdout {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, _buf: crate::mm::UserBuffer) -> usize {
        warn!("kernel #0", "Stdout: not readable, kill this call");
        exit_current_and_run_next(-10);
        0
    }
    fn write(&self, user_buf: crate::mm::UserBuffer) -> usize {
        for buffer in user_buf.buffers.iter() {
            print!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        user_buf.len()
    }
}