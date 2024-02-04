//! File system in os

use crate::mm::UserBuffer;

mod inode;
mod stdio;
pub use inode::{ open_file, OpenFlags, list_apps};
pub use stdio::{Stdin, Stdout, Stderr};

/// File trait
pub trait File: Send + Sync {
    /// If readable
    fn readable(&self) -> bool;
    /// If writable
    fn writable(&self) -> bool;
    /// Read file to `UserBuffer`
    fn read(&self, buf: UserBuffer) -> usize;
    /// Write file from `UserBuffer`
    fn write(&self, buf: UserBuffer) -> usize;
}