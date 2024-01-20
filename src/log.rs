use core::fmt;

pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Level {
    pub fn from_str(level: &str) -> Level {
        match level {
            "error" => Level::Error,
            "warn" => Level::Warn,
            "info" => Level::Info,
            "debug" => Level::Debug,
            "trace" => Level::Trace,
            _ => Level::Info,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Info => "INFO",
            Level::Debug => "DEBUG",
            Level::Trace => "TRACE",
        }
    }

    pub fn to_i32(&self) -> i32 {
        match self {
            Level::Error => 1,
            Level::Warn => 2,
            Level::Info => 3,
            Level::Debug => 4,
            Level::Trace => 5,
        }
    }
}

pub fn log(level: Level, mark: &str, args: fmt::Arguments) {
    let log_level_option = option_env!("LOG");
    let log_level = Level::from_str(log_level_option.unwrap_or("info"));
    if level.to_i32() <= log_level.to_i32() {
        match level {
            Level::Error => {
                println!("\x1b[31m[{}/{}]: {}\x1b[0m", level.to_str(), mark, args);
            }
            Level::Warn => {
                println!("\x1b[33m[{}/{}]: {}\x1b[0m", level.to_str(), mark, args);
            }
            Level::Info => {
                println!("\x1b[34m[{}/{}]: {}\x1b[0m", level.to_str(), mark, args);
            }
            Level::Debug => {
                println!("\x1b[32m[{}/{}]: {}\x1b[0m", level.to_str(), mark, args);
            }
            Level::Trace => {
                println!("\x1b[90m[{}/{}]: {}\x1b[0m", level.to_str(), mark, args);
            }
        }
    }
}

#[macro_export]
macro_rules! error {
    ($mark:literal, $str: literal $(, $($arg:tt)+)?) => {
        $crate::log::log(crate::log::Level::Error, $mark, format_args!($str $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! warn {
    ($num:literal, $str: literal $(, $($arg:tt)+)?) => {
        $crate::log::warn(crate::log::Level::Warn, $num, format_args!($str $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! info {
    ($num:literal, $str: literal $(, $($arg:tt)+)?) => {
        $crate::log::info(crate::log::Level::Info, $num, format_args!($str $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! debug {
    ($num:literal, $str: literal $(, $($arg:tt)+)?) => {
        $crate::log::debug(crate::log::Level::Debug, $num, format_args!($str $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! trace {
    ($num:literal, $str: literal $(, $($arg:tt)+)?) => {
        $crate::log::trace(crate::log::Level::Trace, $num, format_args!($str $(, $($arg)+)?));
    }
}