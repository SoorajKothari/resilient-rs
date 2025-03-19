use std::time::Duration;

/// Defines the retry strategy to use when scheduling retry attempts.
///
/// This enum specifies how delays between retries are calculated.
#[derive(Debug)]
pub enum RetryStrategy {
    /// A linear retry strategy where the delay between retries remains constant.
    ///
    /// For example, if the delay is set to 2 seconds, each retry will wait exactly 2 seconds.
    Linear,
    /// An exponential backoff strategy where the delay increases exponentially with each retry.
    ///
    /// For example, with a base delay of 2 seconds, retries might wait 2s, 4s, 8s, etc.
    ExponentialBackoff,
    /// A Fibonacci backoff strategy where the delay between retries follows the Fibonacci sequence.
    ///
    /// In this strategy, each delay is the sum of the two preceding delays, typically starting with
    /// a base unit (e.g., 1 second). For example, if the base delay is 1 second, the retry delays
    /// would be 1s, 1s, 2s, 3s, 5s, 8s, 13s, etc. This provides a gentler increase compared to
    /// exponential backoff, balancing retry frequency and resource usage.
    ///
    /// ### Example
    /// - Retry 1: 1 second
    /// - Retry 2: 1 second
    /// - Retry 3: 2 seconds
    /// - Retry 4: 3 seconds
    /// - Retry 5: 5 seconds
    /// - And so on...
    FibonacciBackoff,
    /// An arithmetic progression strategy where the delay increases linearly based on a coefficient.
    ///
    /// For example, with a coefficient of 3 and a base delay of 1s:
    /// - Retry 1: 3s
    /// - Retry 2: 6s
    /// - Retry 3: 9s
    /// - And so on...
    ArithmeticProgression { coefficient: usize },
}
/// Configuration for retrying operations.
///
/// This struct defines the parameters for retrying an operation, including
/// the maximum number of attempts, the delay between retries, and the retry strategy.

impl RetryStrategy {
    /// Calculates the delay duration for a specific retry attempt based on the retry strategy.
    ///
    /// # Arguments
    /// * `base_delay` - The base duration to use as the starting point for delay calculations.
    /// * `attempt` - The current attempt number (1-based index for retries).
    ///
    /// # Returns
    /// A `Duration` representing the time to wait before the next retry attempt.
    pub(crate) fn calculate_delay(&self, base_delay: Duration, attempt: usize) -> Duration {
        match self {
            RetryStrategy::Linear => base_delay,
            RetryStrategy::ExponentialBackoff => {
                if attempt == 0 {
                    base_delay
                } else {
                    base_delay * 2u32.pow((attempt - 1) as u32)
                }
            }
            RetryStrategy::FibonacciBackoff => {
                if attempt < 2 {
                    base_delay
                } else {
                    let mut prev = base_delay;
                    let mut curr = base_delay;
                    for _ in 2..=attempt {
                        let next = prev + curr;
                        prev = curr;
                        curr = next;
                    }
                    curr
                }
            }
            RetryStrategy::ArithmeticProgression { coefficient } => {
                base_delay * (*coefficient as u32 * attempt as u32)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_linear_strategy() {
        let base_delay = Duration::from_secs(2);
        let linear = RetryStrategy::Linear;

        // Test that Linear strategy returns a constant delay
        assert_eq!(
            linear.calculate_delay(base_delay, 0),
            Duration::from_secs(2)
        ); // Initial attempt
        assert_eq!(
            linear.calculate_delay(base_delay, 1),
            Duration::from_secs(2)
        ); // First retry
        assert_eq!(
            linear.calculate_delay(base_delay, 2),
            Duration::from_secs(2)
        ); // Second retry
        assert_eq!(
            linear.calculate_delay(base_delay, 3),
            Duration::from_secs(2)
        ); // Third retry
    }

    #[test]
    fn test_exponential_backoff_strategy() {
        let base_delay = Duration::from_secs(2);
        let expo = RetryStrategy::ExponentialBackoff;

        // Test that ExponentialBackoff increases delay exponentially
        assert_eq!(expo.calculate_delay(base_delay, 0), Duration::from_secs(2));
        assert_eq!(expo.calculate_delay(base_delay, 1), Duration::from_secs(2));
        assert_eq!(expo.calculate_delay(base_delay, 2), Duration::from_secs(4));
        assert_eq!(expo.calculate_delay(base_delay, 3), Duration::from_secs(8));
        assert_eq!(expo.calculate_delay(base_delay, 4), Duration::from_secs(16));
    }

    #[test]
    fn test_exponential_backoff_strategy_mill() {
        let base_delay = Duration::from_millis(2000); // Start from 2000ms (2s)
        let expo = RetryStrategy::ExponentialBackoff;

        // Test that ExponentialBackoff increases delay exponentially (milliseconds)
        assert_eq!(
            expo.calculate_delay(base_delay, 0),
            Duration::from_millis(2000)
        ); // Initial attempt
        assert_eq!(
            expo.calculate_delay(base_delay, 1),
            Duration::from_millis(2000)
        ); // 2^0 * 2000ms
        assert_eq!(
            expo.calculate_delay(base_delay, 2),
            Duration::from_millis(4000)
        ); // 2^1 * 2000ms
        assert_eq!(
            expo.calculate_delay(base_delay, 3),
            Duration::from_millis(8000)
        ); // 2^2 * 2000ms
        assert_eq!(
            expo.calculate_delay(base_delay, 4),
            Duration::from_millis(16000)
        ); // 2^3 * 2000ms
    }

    #[test]
    fn test_fibonacci_backoff_strategy() {
        let base_delay = Duration::from_secs(1);
        let fib = RetryStrategy::FibonacciBackoff;

        assert_eq!(fib.calculate_delay(base_delay, 0), Duration::from_secs(1));
        assert_eq!(fib.calculate_delay(base_delay, 1), Duration::from_secs(1));
        assert_eq!(fib.calculate_delay(base_delay, 2), Duration::from_secs(2));
        assert_eq!(fib.calculate_delay(base_delay, 3), Duration::from_secs(3));
        assert_eq!(fib.calculate_delay(base_delay, 4), Duration::from_secs(5));
        assert_eq!(fib.calculate_delay(base_delay, 5), Duration::from_secs(8));
    }

    #[test]
    fn test_fibonacci_backoff_strategy_millis() {
        let base_delay = Duration::from_millis(2000); // 2000ms = 2s
        let fib = RetryStrategy::FibonacciBackoff;

        assert_eq!(
            fib.calculate_delay(base_delay, 0),
            Duration::from_millis(2000)
        ); // 1st: 2000ms
        assert_eq!(
            fib.calculate_delay(base_delay, 1),
            Duration::from_millis(2000)
        ); // 2nd: 2000ms
        assert_eq!(
            fib.calculate_delay(base_delay, 2),
            Duration::from_millis(4000)
        ); // 3rd: 4000ms
        assert_eq!(
            fib.calculate_delay(base_delay, 3),
            Duration::from_millis(6000)
        ); // 4th: 6000ms
        assert_eq!(
            fib.calculate_delay(base_delay, 4),
            Duration::from_millis(10000)
        ); // 5th: 10000ms
        assert_eq!(
            fib.calculate_delay(base_delay, 5),
            Duration::from_millis(16000)
        ); // 6th: 16000ms
    }

    #[test]
    fn test_arithmetic_progression_strategy() {
        let base_delay = Duration::from_secs(2);
        let ap = RetryStrategy::ArithmeticProgression { coefficient: 3 };

        assert_eq!(ap.calculate_delay(base_delay, 1), Duration::from_secs(6));
        assert_eq!(ap.calculate_delay(base_delay, 2), Duration::from_secs(12));
        assert_eq!(ap.calculate_delay(base_delay, 3), Duration::from_secs(18));
    }
}
