use slow5lib_sys::{
    slow5_log_level_opt_SLOW5_LOG_DBUG, slow5_log_level_opt_SLOW5_LOG_ERR,
    slow5_log_level_opt_SLOW5_LOG_INFO, slow5_log_level_opt_SLOW5_LOG_OFF,
    slow5_log_level_opt_SLOW5_LOG_VERB, slow5_log_level_opt_SLOW5_LOG_WARN,
};

enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Verbose,
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

fn slow5_set_log_level(lvl: LogLevel) {
    let slow5_lvl = lvl.to_slow5_log_lvl();
    unsafe { slow5lib_sys::slow5_set_log_level(slow5_lvl) }
}
