use tracing;
use tracing_subscriber;

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {{
        let message = format!($($arg)*);
        tracing::trace!("{}", message);
    }};
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        let message = format!($($arg)*);
        tracing::info!("{}", message);
    }};
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        let message = format!($($arg)*);
        tracing::debug!("{}", message);
    }};
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        let message = format!($($arg)*);
        tracing::warn!("{}", message);
    }};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        let message = format!($($arg)*);
        tracing::error!("{}", message);
    }};
}

#[macro_export]
macro_rules! fatal_end_exit {
    ($($arg:tt)*) => {{
        let message = format!($($arg)*);
        tracing::error!("{}", message);
        std::process::exit(1); //or panic!
    }};
}

pub(crate) fn setup_logging(log_level: &str) {
    let log_level = match log_level.to_lowercase().as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO, // default
    };

    tracing_subscriber::fmt()
        .json()
        .with_max_level(log_level)
        .with_thread_ids(true)
        //.with_thread_names(true)  // If you use named threads
        //.with_file(true)          // Adds source file path
        //.with_line_number(true)   // Adds line number
        .init()
}

/*#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        let timestamp = $crate::utils::logging::timestamp();
        let message = format!("{} [INFO] {}\n", timestamp, format_args!($($arg)*));
        let mut stdout = std::io::stdout();
        let _ = std::io::Write::write_all(&mut stdout, message.as_bytes());
        let _ = std::io::Write::flush(&mut stdout);
    }};
}

pub fn timestamp() -> String {
    crate::utils::datetime::now()
        .format("%M:%S%.3f").to_string()  // MM:SS.sss to have only significant time part
}
*/
