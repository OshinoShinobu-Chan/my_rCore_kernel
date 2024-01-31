mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use address::{StepByOne, VPNRange};
pub use frame_allocator::{frame_alloc, FrameTracker};
pub use memory_set::remap_test;
pub use memory_set::{MapPermission, MemorySet, KERNEL_SPACE};
use page_table::{PTEFlags, PageTable};
pub use page_table::{translated_byte_buffer, PageTableEntry, translate_to_str, translated_refmut};

pub fn init() {
    heap_allocator::init_heap();
    info!("kernel #0", "heap allocator initialized");
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}