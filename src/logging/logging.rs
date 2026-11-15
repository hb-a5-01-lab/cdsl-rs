//! File-based logging for CDSL instruction execution.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use chrono::{DateTime, Local};

use crate::custom_types::Instruction;

// Global log file handle protected by a mutex for thread-safe access.
static LOG_FILE: Mutex<Option<File>> = Mutex::new(None);

/// Represents a single recorded log entry.
#[derive(Debug, Clone)]
pub struct Log {
    /// Timestamp when the event occurred.
    pub timestamp: DateTime<Local>,
    /// Whether the instruction executed successfully.
    pub success: bool,
    /// String representation of the instruction.
    pub instruction_string: String,
    /// Informational or error message accompanying the log.
    pub message: String,
}

/// Initializes the log file in `~/.local/share/cdsl/cdsl.log`.
///
/// Creates parent directories if they do not exist. Should be called once at startup.
pub fn init() -> Result<(), String> {
    let mut file_guard = LOG_FILE
        .lock()
        .map_err(|e| format!("lock log file mutex: {}", e))?;

    // Already initialized.
    if file_guard.is_some() {
        return Ok(());
    }

    let home_dir = dirs::home_dir().ok_or_else(|| "get home directory: not found".to_string())?;

    // Ensure the log directory exists.
    let log_dir: PathBuf = home_dir.join(".local").join("share").join("cdsl");
    if let Err(e) = fs::create_dir_all(&log_dir) {
        return Err(format!("create log directory: {}", e));
    }

    // Open or create the log file in append mode.
    let log_path = log_dir.join("cdsl.log");
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| format!("open log file: {}", e))?;

    *file_guard = Some(file);
    Ok(())
}

/// Flushes and closes the active log file.
///
/// Should be called during application shutdown.
pub fn close() -> Result<(), String> {
    let mut file_guard = LOG_FILE
        .lock()
        .map_err(|e| format!("lock log file mutex: {}", e))?;

    if let Some(mut file) = file_guard.take() {
        let _ = file.flush();
    }

    Ok(())
}

/// Formats a log entry into a standardized single-line string:
/// `[timestamp][instruction][SUCCESS/FAILURE][message]`
pub fn format_entry(entry: &Log) -> String {
    let status = if entry.success { "SUCCESS" } else { "FAILURE" };

    format!(
        "[{}][{}][{}][{}]",
        entry.timestamp.format("%Y-%m-%dT%H:%M:%S%.3f%:z"),
        entry.instruction_string,
        status,
        entry.message
    )
}

/// Writes a timestamped log entry to the active log file.
pub fn log_it(success: bool, message: &str, instruction: &Instruction) -> Result<(), String> {
    let mut file_guard = LOG_FILE
        .lock()
        .map_err(|e| format!("lock log file mutex: {}", e))?;

    let file = match file_guard.as_mut() {
        Some(f) => f,
        None => return Err("logging not initialized: call Init first".to_string()),
    };

    let entry = Log {
        timestamp: Local::now(),
        success,
        instruction_string: instruction.join(" "),
        message: message.to_string(),
    };

    let line = format_entry(&entry);
    writeln!(file, "{}", line).map_err(|e| format!("write log entry: {}", e))?;
    file.flush()
        .map_err(|e| format!("flush log entry: {}", e))?;

    Ok(())
}
