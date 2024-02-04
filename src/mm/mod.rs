mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

use     address::VPNRange;
pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum, StepByOne};
pub use frame_allocator::{frame_alloc, frame_dealloc, FrameTracker};
pub use memory_set::{MapPermission, MemorySet, KERNEL_SPACE, remap_test, kernel_token};
use     page_table::PTEFlags;
pub use page_table::{translated_byte_buffer, PageTableEntry, translate_to_str, translated_refmut,
                    UserBuffer, PageTable};

pub fn init() {
    heap_allocator::init_heap();
    info!("kernel #0", "heap allocator initialized");
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}