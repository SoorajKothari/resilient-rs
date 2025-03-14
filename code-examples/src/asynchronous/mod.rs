use async_std::net::TcpStream;
use std::io::Error;

use async_std::io::{ReadExt, WriteExt};
use async_std::task::sleep;
use resilient_rs::asynchronous::{execute_with_fallback, retry, retry_with_exponential_backoff};
use resilient_rs::config::{ExecConfig, RetryConfig};
use std::time::Duration;

async fn send() -> Result<String, Error> {
    let mut stream = TcpStream::connect("example.com:80").await?;
    let request = "GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n";
    stream.write_all(request.as_bytes()).await?;
    stream.flush().await?;
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;
    let response = String::from_utf8_lossy(&buffer);
    let is_success = response.starts_with("HTTP/1.1 200 OK");
    Ok(is_success.to_string())
}

// Example 1: Simple async retry
pub async fn example_async_retry() {
    let retry_config = RetryConfig::default();

    let result = retry(|| async { send().await }, &retry_config).await;

    match result {
        Ok(success) => println!("Success: {}", success),
        Err(error) => println!("Failed: {}", error),
    }
}

// Example 2: Async retry with exponential backoff and condition
pub async fn example_async_exponential_with_condition() {
    let should_retry = |error: &Error| error.to_string().contains("not found");

    let retry_config = RetryConfig {
        max_attempts: 4,
        delay: Duration::from_millis(100),
        retry_condition: Some(should_retry),
    };

    let result = retry_with_exponential_backoff(|| async { send().await }, &retry_config).await;

    match result {
        Ok(value) => println!("Success: {}", value),
        Err(error) => println!("Failed: {}", error),
    }
}

// Define slow_operation as a reusable async function
async fn slow_operation() -> Result<String, Box<dyn std::error::Error>> {
    sleep(Duration::from_millis(100)).await;
    Ok("Success".to_string())
}

// Example 3: Execute with timeout and optional fallback
pub async fn example_execute_with_fallback() {
    // Config with fallback
    let config_with_fallback = ExecConfig {
        timeout_duration: Duration::from_millis(50),
        fallback: Some(|| Ok("Fallback result".to_string())),
    };

    // Config without fallback
    let config_without_fallback = ExecConfig {
        timeout_duration: Duration::from_millis(50),
        fallback: None::<fn() -> Result<String, Box<dyn std::error::Error>>>,
    };

    // Test with fallback
    let result_with_fallback = execute_with_fallback(slow_operation(), &config_with_fallback).await;
    match result_with_fallback {
        Ok(value) => println!("With fallback result: {}", value),
        Err(e) => println!("With fallback error: {}", e),
    }

    // Test without fallback
    let result_without_fallback =
        execute_with_fallback(slow_operation(), &config_without_fallback).await;
    match result_without_fallback {
        Ok(value) => println!("Without fallback result: {}", value),
        Err(e) => println!("Without fallback error: {}", e),
    }
}
