use crate::config::RetryConfig;
use log::{info, warn};
use std::thread::sleep;

/// Retries a given operation based on the specified retry configuration.
///
/// # Arguments
/// * `operation` - A closure that returns a `Result<T, E>`. The function will retry this operation if it fails.
/// * `retry_config` - A reference to `RetryConfig` specifying the maximum attempts and delay between retries.
///
/// # Returns
/// * `Ok(T)` if the operation succeeds within the allowed attempts.
/// * `Err(E)` if the operation fails after all retry attempts.
///
/// # Example
/// ```
/// use std::time::Duration;
/// use resilient_rs::config::RetryConfig;
/// use resilient_rs::synchronous::retry::retry;
///
/// let retry_config = RetryConfig { max_attempts: 3, delay: Duration::from_millis(500) };
/// let result: Result<i32, &str> = retry(|| {
///     Err("Temporary failure") // Always fails in this example
/// }, &retry_config);
/// assert!(result.is_err()); // Should be Error after 3 attempts
/// ```
pub fn retry<F, T, E>(mut operation: F, retry_config: &RetryConfig) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut attempts = 0;

    loop {
        match operation() {
            Ok(output) => {
                info!("Operation succeeded after {} attempts", attempts + 1);
                return Ok(output);
            }
            Err(_) if attempts + 1 < retry_config.max_attempts => {
                warn!(
                    "Operation failed (attempt {}/{}), retrying after {:?}...",
                    attempts + 1,
                    retry_config.max_attempts,
                    retry_config.delay
                );
                sleep(retry_config.delay);
            }
            Err(err) => {
                warn!(
                    "Operation failed after {} attempts, giving up.",
                    attempts + 1
                );
                return Err(err);
            }
        }

        attempts += 1;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[test]
    fn test_retry_success() {
        let retry_config = RetryConfig {
            max_attempts: 3,
            delay: Duration::from_millis(10),
        };

        let mut attempts = 0;
        let result = retry(
            || {
                attempts += 1;
                if attempts == 2 {
                    Ok("Success")
                } else {
                    Err("Failure")
                }
            },
            &retry_config,
        );

        assert_eq!(result, Ok("Success"));
        assert_eq!(attempts, 2);
    }

    #[test]
    fn test_retry_exhaustion() {
        let retry_config = RetryConfig {
            max_attempts: 3,
            delay: Duration::from_millis(10),
        };

        let attempts = AtomicUsize::new(0);

        let result: Result<(), &str> = retry(
            || {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err("Failure")
            },
            &retry_config,
        );

        assert_eq!(result, Err("Failure"));
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    fn always_fail() -> Result<&'static str, &'static str> {
        Err("Always fails")
    }

    fn succeed_on_third_attempt() -> Result<&'static str, &'static str> {
        static ATTEMPTS: AtomicUsize = AtomicUsize::new(0);
        let count = ATTEMPTS.fetch_add(1, Ordering::SeqCst);
        if count == 2 {
            Ok("Success")
        } else {
            Err("Failure")
        }
    }

    #[test]
    fn test_retry_with_function() {
        let retry_config = RetryConfig {
            max_attempts: 5,
            delay: Duration::from_millis(10),
        };

        let result = retry(succeed_on_third_attempt, &retry_config);
        assert_eq!(result, Ok("Success"));

        let result = retry(always_fail, &retry_config);
        assert_eq!(result, Err("Always fails"));
    }
}
