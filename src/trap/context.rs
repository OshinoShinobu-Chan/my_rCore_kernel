use riscv::register::sstatus::{self, Sstatus, SPP};

// Struct to save the context when trap occurs
#[repr(C)]
pub struct TrapContext {
    // General registers
    pub x: [usize; 32],
    // CSR sstatus
    pub sstatus: Sstatus,
    // CSR sepc
    pub sepc: usize
    // There's no need to save the CSR stval and scause
}

impl TrapContext {
    // set stack pointer to x_2 reg (sp)
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    // init app context, used to start a new app by reusing __restore
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        debug!("kernel #0", "app_init_context: entry={:#x}, sp={:#x}", entry, sp);
        let mut sstatus:Sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            x:[0; 32], // set all the value of general purpose registers to 0
            sstatus,
            sepc: entry, //set pc to the entry point of app
        };
        cx.set_sp(sp);
        cx
    }
}