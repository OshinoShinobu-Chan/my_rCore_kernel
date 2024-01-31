use alloc::vec::Vec;
use lazy_static::*;
use crate::mm::{KERNEL_SPACE, MapPermission, VirtAddr};
use crate::sync::UPSafeCell;
use crate::config::{TRAMPOLINE, PAGE_SIZE, KERNEL_STACK_SIZE};

// Pid Allocator struct
pub struct PidAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl PidAllocator {
    // Create a new PidAllocator
    pub fn new() -> Self {
        PidAllocator {
            current: 0,
            recycled: Vec::new(),
        }
    }
    // Allocate a pid
    pub fn alloc(&mut self) -> PidHandler {
        if let Some(pid) = self.recycled.pop() {
            PidHandler(pid)
        } else {
            self.current += 1;
            PidHandler(self.current - 1)
        }
    }
    // Recycle a pid
    pub fn dealloc(&mut self, pid: usize) {
        assert!(pid < self.current);
        assert!(
            !self.recycled.contains(&pid),
            "pid {} has already been deallocated!",
            pid
        );
        self.recycled.push(pid);
    }
}

// bind pid lifetime to PidHandler
// when pidhandler is dropped, pid will be deallocated automatically
pub struct PidHandler(pub usize);

// Create a global instance of PidAllocator
lazy_static! {
    pub static ref PID_ALLOCATOR: UPSafeCell<PidAllocator> = 
        unsafe { UPSafeCell::new(PidAllocator::new()) };
}

impl Drop for PidHandler {
    fn drop(&mut self) {
        trace!("kernel #0", "drop pid {}", self.0);
        PID_ALLOCATOR.exclusive_access().dealloc(self.0);
    }
}

pub fn pid_alloc() -> PidHandler {
    PID_ALLOCATOR.exclusive_access().alloc()
}

pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}

// Kernel stack of app
pub struct KernelStack {
    pid: usize,
}

impl KernelStack {
    // Create a kernel stack from pid
    pub fn new(pid_handler: &PidHandler) -> Self {
        let pid = pid_handler.0;
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(pid);
        KERNEL_SPACE.exclusive_access().insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );
        KernelStack { pid: pid_handler.0 }
    }
    #[allow(unused)]
    // Push a value on top of kernel stack
    pub fn push_on_top<T>(&self, value: T) -> *mut T
    where
        T: Sized,
    {
        let kernel_stack_top = self.get_top();
        let ptr_mut = (kernel_stack_top - core::mem::size_of::<T>()) as *mut T;
        unsafe {
            *ptr_mut = value;
        }
        ptr_mut
    }
    // Get the address of the top of kernel stack
    pub fn get_top(&self) -> usize {
        let (_, kernel_stack_top) = kernel_stack_position(self.pid);
        kernel_stack_top
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        let (kernel_stack_bottom, _) = kernel_stack_position(self.pid);
        let kernel_stack_bottom_va: VirtAddr = kernel_stack_bottom.into();
        KERNEL_SPACE
            .exclusive_access()
            .remove_area_with_start_vpn(kernel_stack_bottom_va.into());
    }
}