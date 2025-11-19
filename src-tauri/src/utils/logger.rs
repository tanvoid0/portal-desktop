use std::fmt;
use std::sync::Arc;
use chrono::Utc;
use tauri::{AppHandle, Emitter};

/// Log levels matching the frontend logger
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// Logger configuration
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    pub level: LogLevel,
    pub enable_console: bool,
    pub enable_timestamps: bool,
    pub enable_colors: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            enable_console: true,
            enable_timestamps: true,
            enable_colors: true,
        }
    }
}

/// Centralized logger for backend
#[derive(Debug)]
pub struct Logger {
    config: LoggerConfig,
    app_handle: Option<Arc<AppHandle>>,
}

impl Logger {
    /// Create a new logger with default configuration
    pub fn new() -> Self {
        Self {
            config: LoggerConfig::default(),
            app_handle: None,
        }
    }

    /// Set the app handle for emitting events to frontend
    pub fn set_app_handle(&mut self, app_handle: AppHandle) {
        self.app_handle = Some(Arc::new(app_handle));
    }

    /// Create a logger with custom configuration
    pub fn with_config(config: LoggerConfig) -> Self {
        Self { 
            config,
            app_handle: None,
        }
    }

    /// Configure the logger
    pub fn configure(&mut self, config: LoggerConfig) {
        self.config = config;
    }

    /// Log a message at the specified level
    pub fn log(&self, level: LogLevel, context: Option<&str>, message: &str) {
        if level < self.config.level {
            return;
        }

        let timestamp = if self.config.enable_timestamps {
            Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string()
        } else {
            String::new()
        };

        let context_str = context.map(|c| format!("[{}]", c)).unwrap_or_default();
        let level_str = if self.config.enable_colors {
            Self::colorize_level(level)
        } else {
            level.to_string()
        };

        let log_line = if self.config.enable_timestamps {
            format!("[{}] [{}] {}{}", timestamp, level_str, context_str, message)
        } else {
            format!("[{}] {}{}", level_str, context_str, message)
        };

        if self.config.enable_console {
            match level {
                LogLevel::Debug | LogLevel::Info => {
                    println!("{}", log_line);
                }
                LogLevel::Warn | LogLevel::Error => {
                    eprintln!("{}", log_line);
                }
            }
        }

        // Emit to frontend if app handle is available
        if let Some(app_handle) = &self.app_handle {
            let level_str = level.to_string();
            let payload = serde_json::json!({
                "level": level_str,
                "context": context,
                "message": message,
                "timestamp": timestamp
            });
            
            if let Err(e) = app_handle.emit("backend-log", payload) {
                eprintln!("Failed to emit backend log event: {}", e);
            }
        }
    }

    /// Log at debug level
    pub fn debug(&self, context: Option<&str>, message: &str) {
        self.log(LogLevel::Debug, context, message);
    }

    /// Log at info level
    pub fn info(&self, context: Option<&str>, message: &str) {
        self.log(LogLevel::Info, context, message);
    }

    /// Log at warn level
    pub fn warn(&self, context: Option<&str>, message: &str) {
        self.log(LogLevel::Warn, context, message);
    }

    /// Log at error level
    pub fn error(&self, context: Option<&str>, message: &str) {
        self.log(LogLevel::Error, context, message);
    }

    /// Create a scoped logger for a specific context
    pub fn scoped(&self, context: &'static str) -> ScopedLogger {
        ScopedLogger {
            logger: self,
            context,
        }
    }

    /// Colorize log level for terminal output
    fn colorize_level(level: LogLevel) -> String {
        // ANSI color codes
        match level {
            LogLevel::Debug => "\x1b[36mDEBUG\x1b[0m".to_string(), // Cyan
            LogLevel::Info => "\x1b[32mINFO\x1b[0m".to_string(),   // Green
            LogLevel::Warn => "\x1b[33mWARN\x1b[0m".to_string(),   // Yellow
            LogLevel::Error => "\x1b[31mERROR\x1b[0m".to_string(),  // Red
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

/// Scoped logger for a specific context
pub struct ScopedLogger<'a> {
    logger: &'a Logger,
    context: &'static str,
}

impl<'a> ScopedLogger<'a> {
    pub fn debug(&self, message: &str) {
        self.logger.debug(Some(self.context), message);
    }

    pub fn info(&self, message: &str) {
        self.logger.info(Some(self.context), message);
    }

    pub fn warn(&self, message: &str) {
        self.logger.warn(Some(self.context), message);
    }

    pub fn error(&self, message: &str) {
        self.logger.error(Some(self.context), message);
    }
}

/// Global logger instance
static GLOBAL_LOGGER: std::sync::OnceLock<std::sync::Mutex<Logger>> = std::sync::OnceLock::new();

/// Initialize the global logger
pub fn init_logger(config: Option<LoggerConfig>) {
    let logger = if let Some(cfg) = config {
        Logger::with_config(cfg)
    } else {
        Logger::new()
    };
    GLOBAL_LOGGER.set(std::sync::Mutex::new(logger)).expect("Logger already initialized");
}

/// Set app handle for the global logger
pub fn set_app_handle(app_handle: AppHandle) {
    if let Some(logger) = GLOBAL_LOGGER.get() {
        if let Ok(mut logger) = logger.lock() {
            logger.set_app_handle(app_handle);
        }
    }
}

/// Get the global logger instance
pub fn logger() -> std::sync::MutexGuard<'static, Logger> {
    GLOBAL_LOGGER.get_or_init(|| std::sync::Mutex::new(Logger::new())).lock().unwrap()
}

/// Convenience macros for logging
#[macro_export]
macro_rules! log_debug {
    ($context:expr, $($arg:tt)*) => {
        {
            let logger = $crate::utils::logger::logger();
            logger.debug(Some($context), &format!($($arg)*));
        }
    };
    ($($arg:tt)*) => {
        {
            let logger = $crate::utils::logger::logger();
            logger.debug(None, &format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! log_info {
    ($context:expr, $($arg:tt)*) => {
        {
            let logger = $crate::utils::logger::logger();
            logger.info(Some($context), &format!($($arg)*));
        }
    };
    ($($arg:tt)*) => {
        {
            let logger = $crate::utils::logger::logger();
            logger.info(None, &format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($context:expr, $($arg:tt)*) => {
        {
            let logger = $crate::utils::logger::logger();
            logger.warn(Some($context), &format!($($arg)*));
        }
    };
    ($($arg:tt)*) => {
        {
            let logger = $crate::utils::logger::logger();
            logger.warn(None, &format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($context:expr, $($arg:tt)*) => {
        {
            let logger = $crate::utils::logger::logger();
            logger.error(Some($context), &format!($($arg)*));
        }
    };
    ($($arg:tt)*) => {
        {
            let logger = $crate::utils::logger::logger();
            logger.error(None, &format!($($arg)*));
        }
    };
}

