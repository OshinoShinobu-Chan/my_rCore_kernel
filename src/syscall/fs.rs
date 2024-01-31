// filesystem-related syscalls

#[allow(deprecated)]
use sbi_rt::legacy::console_getchar;

use crate::mm::translated_byte_buffer;
use crate::task::{current_user_token, suspend_and_run_next};

const FD_STDOUT: usize = 1;
const FD_STDIN: usize = 0;

// write buf of length `len` to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel#0", "Sys_write is called with fd = {}, buf = {}, len = {}",
            fd, buf as usize, len);
    match fd {
        FD_STDOUT => {
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

#[allow(deprecated)]
// read buf of length `len` from a file with `fd`, only support size = 1 now
pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDIN => {
            assert_eq!(len, 1, "Only support len = 1 in sys_read!");
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
            let ch =  c as u8;
            let mut buffers = translated_byte_buffer(current_user_token(), buf, len);
            unsafe {
                buffers[0].as_mut_ptr().write_volatile(ch);
            }
            1
        }
        _ => {
            panic!("Unsupported fd in sys_read!");
        }
    }
}