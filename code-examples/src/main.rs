use crate::synchronous::{
    example_exponential_backoff, example_retry_with_condition, example_simple_retry,
};

mod asynchronous;
mod synchronous;

// Main function to run both examples
fn sync_examples() {
    println!("Running simple retry example:");
    example_simple_retry();

    println!("\nRunning exponential backoff example:");
    example_exponential_backoff();

    println!("\nRunning retry with condition example:");
    example_retry_with_condition();
}

fn main() {
    sync_examples();
}
