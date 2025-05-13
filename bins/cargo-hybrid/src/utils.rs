//! cargo-hybrid cli utils

/// Initialize the logger with a nice formatted output
pub fn init_logger() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt::fmt().with_env_filter(filter).with_target(false).init();
}
