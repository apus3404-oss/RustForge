// src/engine/cancellation.rs
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;

/// Token for cancelling workflow execution
#[derive(Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
    notify: Arc<Notify>,
}

impl CancellationToken {
    /// Create a new cancellation token
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Cancel the operation
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    /// Check if cancellation was requested
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Wait until cancellation is requested
    pub async fn cancelled(&self) {
        // If already cancelled, return immediately
        if self.is_cancelled() {
            return;
        }

        // Wait for notification
        self.notify.notified().await;
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancellation_token_initial_state() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn test_cancellation_token_cancel() {
        let token = CancellationToken::new();
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_cancellation_token_clone() {
        let token = CancellationToken::new();
        let cloned = token.clone();

        token.cancel();
        assert!(cloned.is_cancelled());
    }

    #[tokio::test]
    async fn test_cancellation_token_wait() {
        let token = CancellationToken::new();
        let token_clone = token.clone();

        let wait_task = tokio::spawn(async move {
            token_clone.cancelled().await;
        });

        // Give the task time to start waiting
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        token.cancel();

        // Wait should complete quickly after cancel
        let result = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            wait_task
        ).await;

        assert!(result.is_ok());
    }
}
