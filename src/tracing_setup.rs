use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn setup_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    let log_directory = std::env::var("LOG_DIRECTORY").unwrap_or_else(|_| "./logs".to_string());
    let log_file = std::env::var("LOG_FILE").unwrap_or_else(|_| "tracing.log".to_string());

    let file_appender = tracing_appender::rolling::daily(&log_directory, &log_file);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .init();

    guard
}
