// filesystem-related syscalls
#[allow(deprecated)]
use crate::fs::{open_file, OpenFlags};
use crate::mm::{translate_to_str, translated_byte_buffer, UserBuffer};
use crate::task::{current_task, current_user_token};

#[allow(unused)]
const FD_STDERR: usize = 2;

// write buf of length `len` to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel #0", "Sys_write is called with fd = {}, buf = {}, len = {}",
            fd, buf as usize, len);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        debug!("kernel #0", "Sys_write: fd out of range");
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable(){
            debug!("kernel #0", "Sys_write: file not writable");
            return -1;
        }
        trace!("kernel #0", "Sys_write: file opened");
        let file = file.clone();
        // release curernt task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        debug!("kernel #0", "Sys_write: fd not opened");
        -1
    }
}

#[allow(deprecated)]
// read buf of length `len` from a file with `fd`, only support size = 1 now
pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel#0", "Sys_read is called with fd = {}, buf = {}, len = {}",
            fd, buf as usize, len);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        debug!("kernel #0", "Sys_read: fd out of range");
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.readable(){
            debug!("kernel #0", "Sys_read: file not readable");
            return -1;
        }
        let file = file.clone();
        // release curernt task TCB manually to avoid multi-borrow
        drop(inner);
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        debug!("kernel #0", "Sys_read: fd not opened");
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel #0", "Sys_open is called with path = {:?}, flags = {}", path, flags);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translate_to_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        trace!("kernel #0", "Sys_open: fd = {}", fd);
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        debug!("kernel #0", "Sys_open: open file failed");
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel#0", "Sys_close is called with fd = {}", fd);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() || inner.fd_table[fd].is_none() {
        debug!("kernel #0", "Sys_close: fd out of range or not opened");
        return -1;
    }
    inner.fd_table[fd].take();
    0
}