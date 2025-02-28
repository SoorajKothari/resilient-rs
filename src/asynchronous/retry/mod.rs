use crate::config::RetryConfig;
use log::{info, warn};
use tokio::time::sleep;

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
}
