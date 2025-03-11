use crate::config::{ExecConfig, RetryConfig};
use async_std::future::{TimeoutError, timeout};
use async_std::task::sleep;
use log::{error, info, warn};
use std::error::Error;

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
pub async fn retry<F, Fut, T, E>(mut operation: F, retry_config: &RetryConfig<E>) -> Result<T, E>
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
            Err(err) if attempts + 1 < retry_config.max_attempts => {
                let should_retry = retry_config.retry_condition.map_or(true, |f| f(&err));
                if should_retry {
                    warn!(
                        "Operation failed (attempt {}/{}), retrying after {:?}...",
                        attempts + 1,
                        retry_config.max_attempts,
                        retry_config.delay
                    );
                    sleep(retry_config.delay).await;
                } else {
                    warn!(
                        "Operation failed (attempt {}/{}), not retryable, giving up.",
                        attempts + 1,
                        retry_config.max_attempts
                    );
                    return Err(err);
                }
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
    retry_config: &RetryConfig<E>,
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
            Err(err) if attempts + 1 < retry_config.max_attempts => {
                let should_retry = retry_config.retry_condition.map_or(true, |f| f(&err));
                if should_retry {
                    warn!(
                        "Operation failed (attempt {}/{}), retrying after {:?}...",
                        attempts + 1,
                        retry_config.max_attempts,
                        delay
                    );
                    sleep(delay).await;
                    delay *= 2;
                } else {
                    warn!(
                        "Operation failed (attempt {}/{}), not retryable, giving up.",
                        attempts + 1,
                        retry_config.max_attempts
                    );
                    return Err(err);
                }
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

pub async fn execute_with_fallback<T>(
    operation: impl Future<Output = Result<T, Box<dyn Error>>>,
    exec_config: &ExecConfig<T>,
) -> Result<T, Box<dyn Error>> {
    match timeout(exec_config.timeout_duration, operation).await {
        Ok(result) => {
            info!("Operation completed before timeout; returning result.");
            result
        }
        Err(e) => {
            if let Some(fallback) = exec_config.fallback {
                warn!("Operation timed out; executing fallback.");
                fallback()
            } else {
                error!("Operation timed out; no fallback provided, returning error.");
                Err(Box::new(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use std::error::Error;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::time::sleep;
    #[derive(Debug, PartialEq, Eq)]
    struct DummyError(&'static str);

    impl std::fmt::Display for DummyError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl Error for DummyError {}

    // Suite for `retry` function
    mod retry_tests {
        use super::*;

        #[test]
        fn test_retry_success_first_try_with_block_on() {
            let config = RetryConfig {
                max_attempts: 3,
                delay: Duration::from_millis(10),
                retry_condition: None,
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

            let result = block_on(retry(operation, &config));
            assert_eq!(result, Ok("success"));
            assert_eq!(*attempts.lock().unwrap(), 1);
        }

        #[tokio::test]
        async fn test_retry_success_first_try() {
            let config = RetryConfig {
                max_attempts: 3,
                delay: Duration::from_millis(10),
                retry_condition: None,
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
                retry_condition: None,
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
                retry_condition: None,
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
        async fn test_retry_fail_first_try_retry_condition_un_match() {
            let config = RetryConfig {
                max_attempts: 3,
                delay: Duration::from_millis(10),
                retry_condition: Some(|e: &DummyError| e.0.contains("transient")),
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

            let result: Result<(), DummyError> = retry(operation, &config).await;
            assert_eq!(result, Err(DummyError("always fail")));
            assert_eq!(*attempts.lock().unwrap(), 1);
        }

        #[tokio::test]
        async fn test_retry_fail_first_try_retry_condition_match() {
            let config = RetryConfig {
                max_attempts: 3,
                delay: Duration::from_millis(10),
                retry_condition: Some(|e: &DummyError| e.0.contains("transient")),
            };

            let attempts = Arc::new(Mutex::new(0));
            let op_attempts = attempts.clone();
            let operation = move || {
                let op_attempts = op_attempts.clone();
                async move {
                    let mut count = op_attempts.lock().unwrap();
                    *count += 1;
                    Err(DummyError("transient"))
                }
            };

            let result: Result<(), DummyError> = retry(operation, &config).await;
            assert_eq!(result, Err(DummyError("transient")));
            assert_eq!(*attempts.lock().unwrap(), 3);
        }
    }

    // Suite for `retry_with_exponential_backoff` function
    mod retry_with_exponential_backoff_tests {
        use super::*;

        #[tokio::test]
        async fn test_retry_with_exponential_backoff_success_first_try() {
            let config = RetryConfig {
                max_attempts: 3,
                delay: Duration::from_millis(10),
                retry_condition: None,
            };

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
                retry_condition: None,
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
                retry_condition: None,
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

        #[tokio::test]
        async fn test_retry_with_exponential_backoff_success_after_failures_with_condition() {
            let config = RetryConfig {
                max_attempts: 5,
                delay: Duration::from_millis(10),
                retry_condition: Some(|e: &DummyError| e.0.contains("405")),
            };

            let attempts = Arc::new(Mutex::new(0));
            let op_attempts = attempts.clone();
            let operation = move || {
                let op_attempts = op_attempts.clone();
                async move {
                    let mut count = op_attempts.lock().unwrap();
                    *count += 1;
                    if *count < 2 {
                        Err(DummyError("temporary fail"))
                    } else {
                        Ok("eventual success")
                    }
                }
            };

            let result = retry_with_exponential_backoff(operation, &config).await;
            assert_eq!(result, Err(DummyError("temporary fail")));
            assert_eq!(*attempts.lock().unwrap(), 1);
        }
    }

    // Suite for `execute_with_timeout` function
    mod execute_with_timeout_tests {
        use super::*;

        #[tokio::test]
        async fn test_execute_with_timeout_success() {
            let config: ExecConfig<String> = ExecConfig {
                timeout_duration: Duration::from_millis(100),
                fallback: None,
            };

            let operation = || async { Ok("success".to_string()) };
            let result = execute_with_fallback(operation(), &config).await;
            assert_eq!(result.unwrap(), "success");
        }

        #[tokio::test]
        async fn test_execute_with_timeout_immediate_failure() {
            let config: ExecConfig<String> = ExecConfig {
                timeout_duration: Duration::from_millis(100),
                fallback: None,
            };

            let operation =
                || async { Err(Box::new(DummyError("immediate failure")) as Box<dyn Error>) };
            let result = execute_with_fallback(operation(), &config).await;
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "immediate failure");
        }

        #[tokio::test]
        async fn test_execute_with_timeout_timeout_no_fallback() {
            let config: ExecConfig<String> = ExecConfig {
                timeout_duration: Duration::from_millis(10),
                fallback: None,
            };

            let operation = || async {
                sleep(Duration::from_millis(50)).await;
                Ok("too slow".to_string())
            };
            let result = execute_with_fallback(operation(), &config).await;
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "future has timed out");
        }

        #[tokio::test]
        async fn test_execute_with_timeout_timeout_with_fallback_success() {
            let mut config: ExecConfig<String> = ExecConfig::new(Duration::from_millis(10));
            config.with_fallback(|| Ok("fallback success".to_string()));

            let operation = || async {
                sleep(Duration::from_millis(50)).await;
                Ok("too slow".to_string())
            };
            let result = execute_with_fallback(operation(), &config).await;
            assert_eq!(result.unwrap(), "fallback success");
        }

        #[tokio::test]
        async fn test_execute_with_timeout_timeout_with_fallback_failure() {
            let mut config: ExecConfig<String> = ExecConfig::new(Duration::from_millis(10));
            config.with_fallback(|| Err(Box::new(DummyError("fallback failed")) as Box<dyn Error>));

            let operation = || async {
                sleep(Duration::from_millis(50)).await;
                Ok("too slow".to_string())
            };
            let result = execute_with_fallback(operation(), &config).await;
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "fallback failed");
        }

        #[tokio::test]
        async fn test_execute_with_timeout_success_near_timeout() {
            let config: ExecConfig<String> = ExecConfig {
                timeout_duration: Duration::from_millis(50),
                fallback: None,
            };

            let operation = || async {
                sleep(Duration::from_millis(40)).await;
                Ok("just in time".to_string())
            };
            let result = execute_with_fallback(operation(), &config).await;
            assert_eq!(result.unwrap(), "just in time");
        }
    }
}
