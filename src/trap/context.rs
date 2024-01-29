use riscv::register::sstatus::{self, Sstatus, SPP};

// Struct to save the context when trap occurs
#[repr(C)]
pub struct TrapContext {
    // General registers
    pub x: [usize; 32],
    // CSR sstatus
    pub sstatus: Sstatus,
    // CSR sepc
    pub sepc: usize,
    // Addr of Page Table
    pub kernel_satp: usize,
    // kernel stack
    pub kernel_sp: usize,
    // Addr of trap_handler function
    pub trap_handler: usize,
}

impl TrapContext {
    // set stack pointer to x_2 reg (sp)
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    // init app context, used to start a new app by reusing __restore
    pub fn app_init_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
            kernel_satp,
            kernel_sp,
            trap_handler,
        };
        cx.set_sp(sp);
        cx
    }
}