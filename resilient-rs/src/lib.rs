/// The `asynchronous` module provides utilities for handling retries and resilience
/// in asynchronous contexts. This includes retry logic and other resilience patterns
/// that are compatible with async/await.
pub mod asynchronous;

/// The `config` module provides configuration structures for retry logic and other
/// resilience patterns. This includes settings like the maximum number of attempts
/// and delay between retries.
pub mod config;

/// The `synchronous` module provides utilities for handling retries and resilience
/// in synchronous contexts. This includes retry logic and other resilience patterns
/// for blocking operations.
pub mod synchronous;
