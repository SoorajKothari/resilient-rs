use std::time::Duration;

#[derive(Debug)]
pub struct RetryConfig {
    pub max_attempts: usize,
    pub delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_attempts: 3,
            delay: Duration::from_secs(2),
        }
    }
}

pub struct RetryWithBackoffConfig {
    pub max_attempts: usize,
    pub delay: Duration,
}
