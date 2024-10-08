#![allow(dead_code)]

use slow5lib_sys::{
    slow5_log_level_opt_SLOW5_LOG_DBUG, slow5_log_level_opt_SLOW5_LOG_ERR,
    slow5_log_level_opt_SLOW5_LOG_INFO, slow5_log_level_opt_SLOW5_LOG_OFF,
    slow5_log_level_opt_SLOW5_LOG_VERB, slow5_log_level_opt_SLOW5_LOG_WARN,
};

/// Set the log level based on desired verbosity.
#[derive(Debug, Clone)]
pub enum LogLevel {
    /// No logs will be generated.
    Off,
    /// Log errors only.
    Error,
    /// Log errors and warnings
    Warn,
    /// Log errors, warnings, and general information
    Info,
    /// Log verbose messages, errors, warnings, and general information
    Verbose,
    /// Log everything
    Debug,
}

impl LogLevel {
    fn to_slow5_log_lvl(&self) -> u32 {
        match self {
            LogLevel::Off => slow5_log_level_opt_SLOW5_LOG_OFF,
            LogLevel::Error => slow5_log_level_opt_SLOW5_LOG_ERR,
            LogLevel::Warn => slow5_log_level_opt_SLOW5_LOG_WARN,
            LogLevel::Info => slow5_log_level_opt_SLOW5_LOG_INFO,
            LogLevel::Verbose => slow5_log_level_opt_SLOW5_LOG_VERB,
            LogLevel::Debug => slow5_log_level_opt_SLOW5_LOG_DBUG,
        }
    }
}

/// Sets the global variable for slow5lib to control the
/// libraries logging verbosity. These represent internal logs
/// of the slow5lib and doesn't interact with logging done by Rust
/// crates.
pub fn slow5_set_log_level(lvl: LogLevel) {
    let slow5_lvl = lvl.to_slow5_log_lvl();
    unsafe { slow5lib_sys::slow5_set_log_level(slow5_lvl) }
}
