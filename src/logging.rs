use std::env;
use std::fs::OpenOptions;
use std::path::PathBuf;

use chrono::Local;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::time::SystemTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn open_log_file() -> std::fs::File {
    let home = env::var("HOME").expect("HOME not set");

    let suffix = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let path = PathBuf::from(home)
        .join(".local/share/zibi")
        .join(format!("zibi.{suffix}.log"));

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create log directory");
    }

    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .expect("failed to open log file")
}

pub fn init() {
    let log_file = open_log_file();

    let file_layer = fmt::layer()
        .with_timer(SystemTime)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .with_writer(log_file);

    let console_layer = fmt::layer()
        .with_timer(SystemTime)
        .with_file(true)
        .with_line_number(true)
        .with_writer(std::io::stdout);

    tracing_subscriber::registry()
        .with(file_layer)
        .with(console_layer)
        .init();
}
