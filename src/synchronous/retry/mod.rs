use std::thread::sleep;
use crate::config::RetryConfig;

pub fn retry<F, T, E>(mut operation: F, retry_config: &RetryConfig) -> Result<T, E>
 where F: FnMut() -> Result<T, E>, {
    let mut attempts = 0;

    while attempts < retry_config.max_attempts {
        match operation() {
            Ok(output) => return Ok(output),
            Err(_) if attempts + 1 < retry_config.max_attempts => sleep(retry_config.delay),
            Err(err) => return Err(err),
        }
        attempts +=1;
    }
    Err("Max attempts cannot be 0")
}