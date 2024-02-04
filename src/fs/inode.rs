use alloc::{sync::Arc, vec::Vec};
use lazy_static::lazy_static;

use crate::{drivers::BLOCK_DEVICE, sync::UPSafeCell};
use easy_fs::{EasyFileSystem, Inode};

use super::File;


/// A Wrapper around a filesystem inode
/// to implement File trait
pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: UPSafeCell<OSInodeInner>,
}
/// The OSInode inner
pub struct OSInodeInner {
    offset: usize,
    inode: Arc<Inode>,
}

lazy_static! {
    /// The root of the file system
    pub static ref ROOT_INODE: Arc<Inode> = {
        let efs = EasyFileSystem::open(BLOCK_DEVICE.clone());
        Arc::new(EasyFileSystem::root_inode(&efs))
    };
}

impl OSInode {
    /// Construct an OS inode from an inode
    pub fn new(readable: bool, writable: bool, inode: Arc<Inode>) -> Self {
        Self {
            readable,
            writable,
            inner: unsafe {
                UPSafeCell::new(OSInodeInner { offset: 0, inode })
            }
        }
    }
    /// Read all data inside an inode into vector
    pub fn read_all(&self) -> Vec<u8> {
        let mut inner = self.inner.exclusive_access();
        let mut buffer = [0u8; 512];
        let mut v: Vec<u8> = Vec::new();
        loop {
            let len = inner.inode.read_at(inner.offset, &mut buffer);
            if len == 0 {
                break;
            }
            inner.offset += len;
            v.extend_from_slice(&buffer[..len])
        }
        v
    }
}

impl File for OSInode {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn read(&self, mut buf: crate::mm::UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_read_size = 0usize;
        for slice in buf.buffers.iter_mut() {
            let read_size = inner.inode.read_at(inner.offset, *slice);
            if read_size == 0 {
                break;
            }
            inner.offset += read_size;
            total_read_size += read_size;
        }
        total_read_size
    }
    fn write(&self, buf: crate::mm::UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter() {
            trace!("kernel #0", "Write to inode at offset {}, content {:?}", inner.offset, *slice);
            trace!("kernel #0", "inode info: {:?}", inner.inode.info());
            let write_size = inner.inode.write_at(inner.offset, *slice);
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }
        total_write_size
    }
}

bitflags! {
    /// Open file flags
    pub struct OpenFlags: u32 {
        /// Read only
        const RDONLY = 0;
        /// Write only
        const WRONLY = 1 << 0;
        /// Read & Write
        const RDWR = 1 << 1;
        /// Allow create
        const CREATE = 1 << 9;
        /// Clear file and return an empty one
        const TRUNC = 1 << 10;
    }
}

impl OpenFlags{
    /// Return (readable, writable)
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}
/// Open file with flag
pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    let (readable, writable) = flags.read_write();
    if flags.contains(OpenFlags::CREATE) {
        if let Some(inode) = ROOT_INODE.find(name) {
            trace!("kernel#0", "File {} already exists and cleared", name);
            // clear file
            inode.clear();
            Some(Arc::new(OSInode::new(readable, writable, inode)))
        } else {
            trace!("kernel#0", "File {} not exists, create it", name);
            // create new file
            ROOT_INODE
                .create(name)
                .map(|inode| Arc::new(OSInode::new(readable, writable, inode)))
        }
    } else {
        ROOT_INODE.find(name).map(|inode| {
            if flags.contains(OpenFlags::TRUNC) {
                inode.clear();
            }
            Arc::new(OSInode::new(readable, writable, inode))
        })
    }
}
/// List all files in the filesystem
pub fn list_apps() {
    println!("/******* APPS *******");
    for app in ROOT_INODE.ls() {
        println!("{}", app);
    }
    println!("********************/");
}