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
/// use resilient_rs::synchronous::retry;
///
/// let retry_config = RetryConfig { max_attempts: 3, delay: Duration::from_millis(500) };
/// let result: Result<i32, &str> = retry(|| {
///     Err("Temporary failure") // Always fails in this example
/// }, &retry_config);
/// assert!(result.is_err()); // Should be Error after 3 attempts
/// ```
/// # Notes
/// - The function logs warnings for failed attempts and final failure.
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

/// Retries a given operation using an exponential backoff strategy.
///
/// # Arguments
///
/// * `operation` - A closure that performs the operation, returning `Result<T, E>`.
/// * `retry_config` - Configuration specifying the initial delay and maximum attempts.
///
/// # Returns
///
/// * `Ok(T)` if the operation succeeds within the allowed attempts.
/// * `Err(E)` if all attempts fail.
///
/// # Example
///
/// ```
/// use resilient_rs::config::RetryConfig;
/// use resilient_rs::synchronous::retry_with_exponential_backoff;
///
/// fn main() {
/// let retry_config = RetryConfig::default();
///     let mut attempts = 0;
///
///     let result = retry_with_exponential_backoff(
///         || {
///             attempts += 1;
///             if attempts < 3 {
///                 Err("Temporary failure")
///             } else {
///                 Ok("Success")
///             }
///         },
///         &retry_config,
///     );
///
///     println!("{:?}", result);
/// }
/// ```
///
/// This example simulates an operation that fails three times before succeeding.
/// The function retries using an exponential backoff strategy.
/// # Notes
/// - The delay is multiplied by 2 after each failed attempt.
/// - The function logs warnings for failed attempts and final failure.
pub fn retry_with_exponential_backoff<F, T, E>(
    mut operation: F,
    retry_config: &RetryConfig,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut attempts = 0;
    let mut delay = retry_config.delay;

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
                    delay
                );
                sleep(delay);
                delay *= 2;
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
    use std::fmt::Error;
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

    #[test]
    fn test_retry_success_on_first_attempt() {
        let retry_config = RetryConfig {
            max_attempts: 3,
            delay: Duration::from_millis(100),
        };

        let result: Result<i32, Error> = retry_with_exponential_backoff(|| Ok(60), &retry_config);
        assert_eq!(result, Ok(60));
    }

    #[test]
    fn test_retry_success_after_failures() {
        let retry_config = RetryConfig {
            max_attempts: 5,
            delay: Duration::from_millis(100),
        };

        static ATTEMPTS: AtomicUsize = AtomicUsize::new(0);

        let result = retry_with_exponential_backoff(
            || {
                if ATTEMPTS.fetch_add(1, Ordering::SeqCst) < 2 {
                    Err("Temporary failure")
                } else {
                    Ok(42)
                }
            },
            &retry_config,
        );

        assert_eq!(result, Ok(42));
        assert_eq!(ATTEMPTS.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_retry_failure_after_max_attempts() {
        let retry_config = RetryConfig {
            max_attempts: 3,
            delay: Duration::from_millis(100),
        };

        static ATTEMPTS: AtomicUsize = AtomicUsize::new(0);

        let result: Result<(), &str> = retry_with_exponential_backoff(
            || {
                ATTEMPTS.fetch_add(1, Ordering::SeqCst);
                Err("Permanent failure")
            },
            &retry_config,
        );

        assert_eq!(result, Err("Permanent failure"));
        assert_eq!(ATTEMPTS.load(Ordering::SeqCst), 3);
    }
}
