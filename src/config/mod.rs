use std::time::Duration;
/// Configuration for retrying operations.
///
/// This struct defines the parameters for retrying an operation, including
/// the maximum number of attempts and the delay between retries.
#[derive(Debug)]
pub struct RetryConfig {
    /// The maximum number of retry attempts.
    ///
    /// This specifies how many times the operation will be retried before
    /// giving up. For example, if `max_attempts` is set to 3, the operation
    /// will be attempted up to 3 times (1 initial attempt + 2 retries).
    pub max_attempts: usize,

    /// The delay between retry attempts.
    ///
    /// This specifies the amount of time to wait between each retry attempt.
    /// For example, if `delay` is set to `Duration::from_secs(2)`, the program
    /// will wait 2 seconds between retries.
    pub delay: Duration,
}
impl Default for RetryConfig {
    /// Provides a default configuration for retrying operations.
    ///
    /// The default configuration includes:
    /// - `max_attempts`: 3 retries
    /// - `delay`: 2 seconds between retries
    ///
    /// This implementation allows you to create a `RetryConfig` with sensible
    /// defaults using `RetryConfig::default()`.
    fn default() -> Self {
        RetryConfig {
            max_attempts: 3,
            delay: Duration::from_secs(2),
        }
    }
}
