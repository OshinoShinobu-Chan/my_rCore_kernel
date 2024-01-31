//ref:: https://github.com/andre-richter/qemu-exit
pub const MEMORY_END: usize = 0x81000000;
pub const CLOCK_FREQ: usize = 12500000;

pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000),
];