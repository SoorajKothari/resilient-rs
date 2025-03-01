use crate::config::RetryConfig;
use log::{info, warn};
use tokio::time::sleep;

/// Retries a given asynchronous operation based on the specified retry configuration.
///
/// # Arguments
/// * `operation` - A closure that returns a `Future` resolving to a `Result<T, E>`. The function will retry this operation if it fails.
/// * `retry_config` - A reference to `RetryConfig` specifying the maximum attempts and delay between retries.
///
/// # Returns
/// * `Ok(T)` if the operation succeeds within the allowed attempts.
/// * `Err(E)` if the operation fails after all retry attempts.
///
/// # Example
/// ```
/// use tokio::time::Duration;
/// use reqwest::Client;
/// use resilient_rs::asynchronous::retry;
/// use resilient_rs::config::RetryConfig;
///
/// async fn fetch_url() -> Result<String, reqwest::Error> {
///   use std::fmt::Error;
/// let client = Client::new();
///   let response = client.get("https://example.com")
///           .send()
///           .await?;
///     Ok(response.status().is_success().to_string())
/// }
///
/// #[tokio::main]
/// async fn main() {
///   let retry_config = RetryConfig::default();
///
///   let result = retry(fetch_url, &retry_config).await;
///   match result {
///     Ok(output) => println!("Operation succeeded: {}", output),
///     Err(err) => println!("Operation failed: {}", err),
///   }
/// }
/// ```
/// # Notes
/// - The function logs warnings for failed attempts and final failure.
pub async fn retry<F, Fut, T, E>(mut operation: F, retry_config: &RetryConfig) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut attempts = 0;

    loop {
        match operation().await {
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
                sleep(retry_config.delay).await;
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

/// Retries an asynchronous operation using exponential backoff.
///
/// This function repeatedly attempts to execute the provided asynchronous operation
/// until it either succeeds or reaches the maximum number of retry attempts.
///
/// # Parameters
/// - `operation`: A function that returns a `Future` resolving to a `Result<T, E>`.
/// - `retry_config`: A reference to a `RetryConfig` struct specifying the delay and maximum attempts.
///
/// # Returns
/// - `Ok(T)`: If the operation succeeds within the allowed retry attempts.
/// - `Err(E)`: If the operation continues to fail after the maximum retry attempts.
///
/// # Behavior
/// - Starts with an initial delay specified in `retry_config.delay`.
/// - On each failure, logs a warning and doubles the delay before retrying.
/// - Stops retrying once `retry_config.max_attempts` is reached.
///
/// # Example
/// ```rust
/// use std::time::Duration;
/// use tokio::time::sleep;
/// use resilient_rs::asynchronous::retry_with_exponential_backoff;
/// use resilient_rs::config::RetryConfig;
///
/// async fn my_operation() -> Result<(), &'static str> {
///     Err("Some error")
/// }
///
/// #[tokio::main]
/// async fn main() {
/// let config = RetryConfig::default();
///
///     let result = retry_with_exponential_backoff(my_operation, &config).await;
///     match result {
///         Ok(_) => println!("Success!"),
///         Err(e) => println!("Failed: {}", e),
///     }
/// }
/// ```
///
/// # Notes
/// - The delay is multiplied by 2 after each failed attempt.
/// - The function logs warnings for failed attempts and final failure.
pub async fn retry_with_exponential_backoff<F, Fut, T, E>(
    mut operation: F,
    retry_config: &RetryConfig,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut attempts = 0;
    let mut delay = retry_config.delay;

    loop {
        match operation().await {
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
                sleep(delay).await;
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
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[derive(Debug, PartialEq, Eq)]
    struct DummyError(&'static str);

    #[tokio::test]
    async fn test_retry_success_first_try() {
        let config = RetryConfig {
            max_attempts: 3,
            delay: Duration::from_millis(10),
        };

        let attempts = Arc::new(Mutex::new(0));

        let op_attempts = attempts.clone();
        let operation = move || {
            let op_attempts = op_attempts.clone();
            async move {
                let mut count = op_attempts.lock().unwrap();
                *count += 1;
                Ok::<_, DummyError>("success")
            }
        };

        let result = retry(operation, &config).await;
        assert_eq!(result, Ok("success"));
        assert_eq!(*attempts.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let config = RetryConfig {
            max_attempts: 5,
            delay: Duration::from_millis(10),
        };

        let attempts = Arc::new(Mutex::new(0));

        let op_attempts = attempts.clone();
        let operation = move || {
            let op_attempts = op_attempts.clone();
            async move {
                let mut count = op_attempts.lock().unwrap();
                *count += 1;
                if *count < 4 {
                    Err(DummyError("temporary failure"))
                } else {
                    Ok("eventual success")
                }
            }
        };

        let result = retry(operation, &config).await;
        assert_eq!(result, Ok("eventual success"));
        assert_eq!(*attempts.lock().unwrap(), 4);
    }

    #[tokio::test]
    async fn test_retry_failure_all_attempts() {
        let config = RetryConfig {
            max_attempts: 3,
            delay: Duration::from_millis(10),
        };

        let attempts = Arc::new(Mutex::new(0));

        let op_attempts = attempts.clone();
        let operation = move || {
            let op_attempts = op_attempts.clone();
            async move {
                let mut count = op_attempts.lock().unwrap();
                *count += 1;
                Err(DummyError("permanent failure"))
            }
        };

        let result: Result<(), DummyError> = retry(operation, &config).await;
        assert_eq!(result, Err(DummyError("permanent failure")));
        assert_eq!(*attempts.lock().unwrap(), config.max_attempts);
    }

    #[tokio::test]
    async fn test_retry_with_exponential_backoff_success_first_try() {
        let config = RetryConfig::default();

        let attempts = Arc::new(Mutex::new(0));

        let op_attempts = attempts.clone();
        let operation = move || {
            let op_attempts = op_attempts.clone();
            async move {
                let mut count = op_attempts.lock().unwrap();
                *count += 1;
                Ok::<_, DummyError>("successful")
            }
        };

        let result = retry_with_exponential_backoff(operation, &config).await;
        assert_eq!(result, Ok("successful"));
        assert_eq!(*attempts.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_retry_with_exponential_backoff_success_after_failures() {
        let config = RetryConfig {
            max_attempts: 5,
            delay: Duration::from_millis(10),
        };

        let attempts = Arc::new(Mutex::new(0));

        let op_attempts = attempts.clone();
        let operation = move || {
            let op_attempts = op_attempts.clone();
            async move {
                let mut count = op_attempts.lock().unwrap();
                *count += 1;
                if *count < 4 {
                    Err(DummyError("temporary fail"))
                } else {
                    Ok("eventual success")
                }
            }
        };

        let result = retry_with_exponential_backoff(operation, &config).await;
        assert_eq!(result, Ok("eventual success"));
        assert_eq!(*attempts.lock().unwrap(), 4);
    }

    #[tokio::test]
    async fn test_retry_with_exponential_backoff_failure_all_attempts() {
        let config = RetryConfig {
            max_attempts: 3,
            delay: Duration::from_millis(10),
        };

        let attempts = Arc::new(Mutex::new(0));

        let op_attempts = attempts.clone();
        let operation = move || {
            let op_attempts = op_attempts.clone();
            async move {
                let mut count = op_attempts.lock().unwrap();
                *count += 1;
                Err(DummyError("always fail"))
            }
        };

        let result: Result<(), DummyError> =
            retry_with_exponential_backoff(operation, &config).await;
        assert_eq!(result, Err(DummyError("always fail")));
        assert_eq!(*attempts.lock().unwrap(), config.max_attempts);
    }
}
