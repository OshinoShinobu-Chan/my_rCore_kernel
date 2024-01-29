use crate::trap::trap_return;


// Struct for tesk context
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct TaskContext {
    // retern address
    ra: usize,
    // kernel stack pointer
    sp: usize,
    // callee saved registers: s0~s11
    s: [usize; 12],
}

impl TaskContext {
    // initialize task context using zero
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
    pub fn goto_trap_return(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}