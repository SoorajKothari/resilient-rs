use crate::asynchronous::{
    circuit_breaker, example_async_exponential_with_condition, example_async_retry,
    example_execute_with_fallback,
};
use crate::synchronous::{
    example_exponential_backoff, example_retry_with_condition, example_simple_retry,
};
use futures::executor::block_on;

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

fn async_examples_with_futures() {
    println!("With futures\n");
    println!("Running async retry example:");
    block_on(example_async_retry());

    println!("\nRunning async exponential backoff with condition example:");
    block_on(example_async_exponential_with_condition());

    println!("\nRunning execute with fallback example:");
    block_on(example_execute_with_fallback());
}

fn async_examples_with_async_std() {
    println!("With Async-Std\n");
    println!("Running async retry example:");
    block_on(example_async_retry());

    println!("\nRunning async exponential backoff with condition example:");
    block_on(example_async_exponential_with_condition());

    println!("\nRunning execute with fallback example:");
    block_on(example_execute_with_fallback());
}

fn async_examples_with_tokio() {
    println!("\nWith tokio\n Coming Soon");
}

fn async_example_circuit_breaker() {
    println!("\nCircuit Breaker");
    block_on(circuit_breaker()).expect("Error");
}

fn main() {
    sync_examples();
    async_examples_with_futures();
    async_examples_with_async_std();
    async_examples_with_tokio();
    async_example_circuit_breaker();
}
