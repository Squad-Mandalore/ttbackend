use std::path::PathBuf;

use chrono::{Duration, NaiveDate, Utc};
use tokio::{fs, io};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn setup_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    let log_directory = std::env::var("LOG_DIRECTORY").unwrap_or_else(|_| String::from("./logs"));
    let log_file = std::env::var("LOG_FILE").unwrap_or_else(|_| String::from("tracing.log"));

    let file_appender = tracing_appender::rolling::daily(log_directory, log_file);
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

pub async fn remove_old_logfiles() -> io::Result<()> {
    let log_directory = std::env::var("LOG_DIRECTORY").unwrap_or_else(|_| String::from("./logs"));
    tracing::debug!("Log directory: {:?}", log_directory);
    let log_file = std::env::var("LOG_FILE").unwrap_or_else(|_| String::from("tracing.log"));
    tracing::debug!("Log file: {:?}", log_file);
    let entries = fs::read_dir(&log_directory).await?;

    let files = extract_date(&log_file, entries).await?;

    remove_old_files(files).await
}

async fn extract_date(
    log_file: &str,
    mut entries: fs::ReadDir,
) -> io::Result<Vec<(PathBuf, NaiveDate)>> {
    let mut files: Vec<(PathBuf, NaiveDate)> = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(date) = path
            .file_name()
            .and_then(|filename| {
                tracing::debug!("Got filename: {:?}", filename);
                filename.to_str()
            })
            .and_then(|filename_str| {
                tracing::debug!("Got filename to str: {:?}", filename_str);
                let strip_prefix = format!("{}.", &log_file);
                filename_str.strip_prefix(&strip_prefix)
            })
            .and_then(|date_str| {
                tracing::debug!("Stripped filename: {:?}", date_str);
                NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
            })
        {
            tracing::debug!("Stripped filename: {:?}", date);
            files.push((path, date));
        } else {
            tracing::warn!("Failed to parse date from filename: {:?}", path);
        }
    }

    files.sort_by_key(|key| key.1);
    Ok(files)
}

async fn remove_old_files(files: Vec<(PathBuf, NaiveDate)>) -> io::Result<()> {
    let today = Utc::now().date_naive();
    for (file_path, date) in files {
        if today.signed_duration_since(date) > Duration::days(30) {
            fs::remove_file(&file_path).await?;
            tracing::info!("Removed: {:?}", file_path);
        }
    }

    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs::File;

    #[tokio::test]
    async fn test_remove_old_logfiles() -> io::Result<()> {
        let log_file_name = String::from("tracing.log");

        let tmp_dir = tempdir()?;

        std::env::set_var("LOG_DIRECTORY", tmp_dir.path().to_str().unwrap());
        std::env::set_var("LOG_FILE", &log_file_name);
        println!("Log directory: {:?}", tmp_dir.path().to_str().unwrap());
        println!("Log file: {:?}", &log_file_name);

        let today = Utc::now().date_naive();

        // Create an old log file (older than 30 days)
        let old_file_date = today - chrono::Duration::days(31);
        let old_file_path = tmp_dir.path().join(format!(
            "{}.{}",
            &log_file_name,
            old_file_date.format("%Y-%m-%d")
        ));
        println!("Old file path: {:?}", old_file_path);
        let _ = File::create(&old_file_path).await?;

        // Create a new log file (newer than 30 days)
        let new_file_date = today - chrono::Duration::days(10);
        let new_file_path = tmp_dir.path().join(format!(
            "{}.{}",
            &log_file_name,
            new_file_date.format("%Y-%m-%d")
        ));
        println!("New file path: {:?}", new_file_path);
        let _ = File::create(&new_file_path).await?;

        // Run the cleanup function
        println!("Running cleanup function");
        remove_old_logfiles().await?;

        // Check results
        assert!(!old_file_path.exists(), "Old file should be deleted");
        assert!(new_file_path.exists(), "New file should not be deleted");

        Ok(())
    }
}
