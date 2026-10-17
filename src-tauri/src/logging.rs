use serde::Serialize;
use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

const MAX_BYTES: u64 = 1024 * 1024;

struct Log {
    path: PathBuf,
    bytes: u64,
    last_window: Option<Value>,
}

impl Log {
    fn open(directory: &Path) -> io::Result<Self> {
        fs::create_dir_all(directory)?;
        let path = directory.join("floatleaf.log");
        let bytes = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?
            .metadata()?
            .len();
        Ok(Self {
            path,
            bytes,
            last_window: None,
        })
    }

    fn write(&mut self, event: &str, details: Value) -> io::Result<()> {
        // Reading ticks reapply the same window state; keep only changes.
        if event == "window" {
            if self.last_window.as_ref() == Some(&details) {
                return Ok(());
            }
            self.last_window = Some(details.clone());
        }
        let line = format!(
            "{}\n",
            json!({
                "timeUnixMs": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
                "event": event,
                "details": details,
            })
        );
        if self.bytes + line.len() as u64 > MAX_BYTES {
            let previous = self.path.with_file_name("floatleaf.previous.log");
            if previous.exists() {
                fs::remove_file(&previous)?;
            }
            fs::rename(&self.path, previous)?;
            self.bytes = 0;
        }
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?
            .write_all(line.as_bytes())?;
        self.bytes += line.len() as u64;
        Ok(())
    }
}

struct Logs {
    directory: PathBuf,
    writer: Mutex<Result<Log, String>>,
}

#[derive(Serialize)]
pub struct LogInfo {
    directory: String,
    version: &'static str,
    error: Option<String>,
}

pub fn setup(app: &AppHandle, data_directory: &Path) {
    let directory = data_directory.join("logs");
    let writer = Mutex::new(Log::open(&directory).map_err(|e| e.to_string()));
    app.manage(Logs { directory, writer });
    record(
        app,
        "start",
        json!({
            "version": env!("CARGO_PKG_VERSION"),
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "debug": cfg!(debug_assertions),
            "pid": std::process::id(),
        }),
    );
}

pub fn record(app: &AppHandle, event: &str, details: Value) {
    let logs = app.state::<Logs>();
    let mut writer = logs.writer.lock().unwrap();
    if let Ok(log) = writer.as_mut()
        && let Err(error) = log.write(event, details)
    {
        eprintln!("日志写入失败：{error}");
        *writer = Err(error.to_string());
    }
}

pub fn info(app: &AppHandle) -> LogInfo {
    let logs = app.state::<Logs>();
    LogInfo {
        directory: logs.directory.to_string_lossy().into_owned(),
        version: env!("CARGO_PKG_VERSION"),
        error: logs.writer.lock().unwrap().as_ref().err().cloned(),
    }
}

pub fn open_directory(app: &AppHandle) -> Result<(), String> {
    let logs = app.state::<Logs>();
    #[cfg(windows)]
    let program = "explorer.exe";
    #[cfg(target_os = "macos")]
    let program = "open";
    std::process::Command::new(program)
        .arg(&logs.directory)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("无法打开日志文件夹：{e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_rotation_keeps_recent_events_and_suppresses_unchanged_window_ticks() {
        let directory = tempfile::tempdir().unwrap();
        let mut log = Log::open(directory.path()).unwrap();
        log.write("window", json!({"visible": false})).unwrap();
        log.write("window", json!({"visible": false})).unwrap();
        assert_eq!(fs::read_to_string(&log.path).unwrap().lines().count(), 1);
        log.write("window", json!({"visible": true})).unwrap();
        assert_eq!(fs::read_to_string(&log.path).unwrap().lines().count(), 2);
        for _ in 0..3 {
            log.write(
                "test",
                json!({"message": "x".repeat(MAX_BYTES as usize / 2)}),
            )
            .unwrap();
        }
        let current = fs::read_to_string(&log.path).unwrap();
        let previous = fs::read_to_string(directory.path().join("floatleaf.previous.log")).unwrap();
        assert!(current.len() < MAX_BYTES as usize && previous.len() < MAX_BYTES as usize);
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
        for line in current.lines().chain(previous.lines()) {
            assert_eq!(
                serde_json::from_str::<Value>(line).unwrap()["event"],
                "test"
            );
        }
    }
}
