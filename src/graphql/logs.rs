use async_graphql::Object;

pub struct Logs;

#[Object]
impl Logs {
    async fn trace(&self, log_message: String) -> String {
        tracing::trace!(log_message);
        log_message
    }

    async fn debug(&self, log_message: String) -> String {
        tracing::debug!(log_message);
        log_message
    }

    async fn info(&self, log_message: String) -> String {
        tracing::info!(log_message);
        log_message
    }

    async fn warn(&self, log_message: String) -> String {
        tracing::warn!(log_message);
        log_message
    }

    async fn error(&self, log_message: String) -> String {
        tracing::error!(log_message);
        log_message
    }
}

impl Default for Logs {
    fn default() -> Self {
        Logs
    }
}
