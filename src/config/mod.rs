use std::time::Duration;

/// Configuration for retrying operations.
///
/// This struct defines the parameters for retrying an operation, including
/// the maximum number of attempts and the delay between retries.
#[derive(Debug)]
pub struct RetryConfig<E> {
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

    /// An optional function to determine if a retry should be attempted.
    ///
    /// This field allows you to specify a custom condition for retrying based on the error type `E`.
    /// It takes a reference to the error (`&E`) and returns a `bool`:
    /// - `true` if the operation should be retried.
    /// - `false` if the operation should not be retried, causing it to fail immediately.
    ///
    /// If set to `None` (the default), all errors will trigger a retry up to `max_attempts`.
    /// If set to `Some(fn)`, only errors for which the function returns `true` will be retried.
    ///
    /// # Examples
    /// ```
    /// use std::time::Duration;
    /// use resilient_rs::config::RetryConfig;
    /// let config = RetryConfig {
    ///     max_attempts: 3,
    ///     delay: Duration::from_secs(1),
    ///     retry_condition: Some(|e: &String| e.contains("transient")),
    /// };
    /// ```
    /// In this example, only errors containing the word "transient" will trigger retries.
    pub retry_condition: Option<fn(&E) -> bool>,
}

impl<E> Default for RetryConfig<E> {
    /// Provides a default configuration for retrying operations.
    ///
    /// The default configuration includes:
    /// - `max_attempts`: 3 retries
    /// - `delay`: 2 seconds between retries
    /// - `should_retry`: `None`, meaning all errors trigger retries
    ///
    /// This implementation allows you to create a `RetryConfig` with sensible
    /// defaults using `RetryConfig::default()`.
    fn default() -> Self {
        RetryConfig {
            max_attempts: 3,
            delay: Duration::from_secs(2),
            retry_condition: None,
        }
    }
}
impl<E> RetryConfig<E> {
    /// Creates a new `RetryConfig` with the specified maximum attempts and delay.
    ///
    /// This constructor initializes a `RetryConfig` with the given `max_attempts` and `delay`,
    /// setting `should_retry` to `None`. When `should_retry` is `None`, all errors will trigger
    /// retries up to the specified `max_attempts`.
    ///
    /// # Arguments
    /// * `max_attempts` - The maximum number of attempts (including the initial attempt).
    /// * `delay` - The duration to wait between retry attempts.
    ///
    /// # Returns
    /// A new `RetryConfig` instance with the provided settings and no retry condition.
    ///
    /// # Examples
    /// ```
    /// use std::time::Duration;
    /// use resilient_rs::config::RetryConfig;
    /// let config = RetryConfig::<String>::new(3, Duration::from_secs(1));
    /// ```
    pub fn new(max_attempts: usize, delay: Duration) -> Self {
        RetryConfig {
            max_attempts,
            delay,
            retry_condition: None,
        }
    }

    /// Sets a custom retry condition and returns the modified `RetryConfig`.
    ///
    /// This method allows you to specify a function that determines whether an operation should
    /// be retried based on the error. It takes ownership of the `RetryConfig`, sets the
    /// `should_retry` field to the provided function, and returns the updated instance.
    /// This enables method chaining in a builder-like pattern.
    ///
    /// # Arguments
    /// * `should_retry` - A function that takes a reference to an error (`&E`) and returns
    ///   `true` if the operation should be retried, or `false` if it should fail immediately.
    ///
    /// # Returns
    /// The updated `RetryConfig` with the specified retry condition.
    ///
    /// # Examples
    /// ```
    /// use std::time::Duration;
    /// use resilient_rs::config::RetryConfig;
    /// let config = RetryConfig::new(3, Duration::from_secs(1))
    ///     .with_retry_condition(|e: &String| e.contains("transient"));
    /// ```
    pub fn with_retry_condition(mut self, retry_condition: fn(&E) -> bool) -> Self {
        self.retry_condition = Some(retry_condition);
        self
    }
}

/// Configuration for executable tasks supporting both synchronous and asynchronous operations.
///
/// This struct defines execution parameters for tasks that may run either synchronously
/// or asynchronously, including a timeout duration and an optional fallback function.
/// It's designed to be passed to functions that handle task execution with support for
/// both execution models.
///
/// # Type Parameters
/// * `T` - The type of the successful result, must implement `Clone`
/// * `E` - The type of the error that may occur during execution
///
#[derive(Debug)]
pub struct ExecConfig<T, E> {
    /// The maximum duration allowed for task execution before timeout.
    ///
    /// This applies to both synchronous and asynchronous operations. For async operations,
    /// this typically integrates with runtime timeout mechanisms.
    pub timeout_duration: Duration,

    /// Optional fallback function to execute if the primary task fails or times out.
    ///
    /// The fallback must be a synchronous function that returns a `Result`. For async
    /// contexts, the execution function is responsible for handling the sync-to-async
    /// transition if needed.
    pub fallback: Option<fn() -> Result<T, E>>,
}

impl<T, E> ExecConfig<T, E>
where
    T: Clone,
{
    /// Creates a new execution configuration with the specified timeout duration.
    ///
    /// Initializes the configuration without a fallback function. This is suitable
    /// for both synchronous and asynchronous task execution scenarios.
    ///
    /// # Arguments
    /// * `timeout_duration` - Maximum execution time for sync or async operations
    ///
    /// # Returns
    /// A new `ExecConfig` instance configured with the given timeout
    pub fn new(timeout_duration: Duration) -> Self {
        ExecConfig {
            timeout_duration,
            fallback: None,
        }
    }

    /// Sets a fallback function to handle task failure or timeout scenarios.
    ///
    /// The fallback is a synchronous function, but can be used in both sync and async
    /// execution contexts. When used with async operations, the executing function
    /// should handle any necessary async adaptation.
    ///
    /// # Arguments
    /// * `fallback` - Synchronous function returning a `Result` with matching types
    pub fn with_fallback(&mut self, fallback: fn() -> Result<T, E>) {
        self.fallback = Some(fallback);
    }
}
