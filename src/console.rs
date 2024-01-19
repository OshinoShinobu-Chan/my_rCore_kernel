use crate::sbi::console_putchar;
use core::fmt::{self, Write};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($str: literal $(, $($tail:tt)+)?) => {
        $crate::console::print(format_args!($str $(, $($tail)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($str: literal $(, $($tail:tt)+)?) => {
        $crate::console::print(format_args!(concat!($str, "\n") $(, $($tail)+)?));
    }
}