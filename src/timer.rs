use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;
use riscv::register::time;


const TICKS_PER_SEC: usize = 100;
const MSEC_PER_SEC: usize = 1000;
const USEC_PER_SEC: usize = 1_0000_000;
#[allow(unused)]
const INF: usize = usize::MAX;

// read the `mtime` register
pub fn get_time() -> usize {
    time::read()
}

// get current time in milliseconds
pub fn get_time_ms() -> usize {
    get_time() / (CLOCK_FREQ / MSEC_PER_SEC)
}

#[allow(unused)]
// get current time in microseconds
pub fn get_time_us() -> usize {
    get_time() / (CLOCK_FREQ / USEC_PER_SEC)
}

// set the next timer interrupt
pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}

#[allow(unused)]
// close timer
pub fn close_timer() {
    set_timer(INF);
}