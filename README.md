<h1 align="center">Resilient-rs</h1>
<div align="center">

<i>A Rust utility library for fault tolerance, including retry strategies, backoff mechanisms, failure handling and much more.</i>
<br>
<br>
<a href="https://discord.com/invite/BymX4aJeEQ"><img src="https://img.shields.io/discord/733027681184251937.svg?style=flat&label=Join%20Community&color=7289DA" alt="Join Community Badge"/></a>
<a href="https://github.com/semicolon-10/resilient-rs/graphs/contributors"><img alt="GitHub contributors" src="https://img.shields.io/github/contributors/semicolon-10/resilient-rs.svg"></a>
[![Crates.io](https://img.shields.io/crates/v/resilient-rs.svg)](https://crates.io/crates/resilient-rs)
[![Downloads](https://img.shields.io/crates/d/resilient-rs)](https://crates.io/crates/resilient-rs)
[![YouTube](https://img.shields.io/badge/YouTube-Semicolon10-red?logo=youtube)](https://www.youtube.com/@Semicolon10)
<br>
<br>
<i>💖 Loved the work? [Subscribe to my YouTube channel](https://www.youtube.com/@Semicolon10) or consider giving this repository a ⭐ to show your support!</i>
</div>


## Features

| **Type**                 | **Feature**        | **Status**              |
|--------------------------|--------------------|-------------------------|
| Synchronous              | Retry              | ✅ Stable               |
| Synchronous              | Retry-with-backoff | 🚧 Under Development    |
| Asynchronous             | Retry              | ✅ Stable               |
| Asynchronous             | Retry-with-backoff | 🚧 Under Development    |
| Asynchronous             | Circuit Breaker    | 🛠️ Planned              |
| Synchronous/Asynchronous | More Examples      | 🛠️ Planned              |

---

## 📦 How to Use `resilient-rs`

Here’s a quick example of how to use the `resilient-rs` crate in your Rust project.

### 1️⃣ Add `resilient-rs` to Your `Cargo.toml`

Add the following line to your `Cargo.toml` file:

```toml
[dependencies]
resilient-rs = "0.1.0" # Replace with the latest version
```

OR

```bash
cargo add resilient-rs
```

#### Synchronous
```rust
use std::time::Duration;
use resilient_rs::config::RetryConfig;
use resilient_rs::synchronous::retry::retry;

fn main() {
  let retry_config = RetryConfig::default();
  let result: Result<i32, &str> = retry(|| {
    Err("Temporary failure")
  }, &retry_config);
  assert!(result.is_err());
}
```

#### Asynchronous
```rust
use tokio::time::Duration;
use log::{info, warn};

async fn example_operation() -> Result<&'static str, &'static str> {
  static mut ATTEMPTS: usize = 0;
  unsafe {
    ATTEMPTS += 1;
    if ATTEMPTS == 3 {
      Ok("Success")
    } else {
      Err("Failure")
    }
  }
}

#[tokio::main]
async fn main() {
  use resilient_rs::asynchronous::retry::retry;
  use resilient_rs::config::RetryConfig;

  let retry_config = RetryConfig {
    max_attempts: 5,
    delay: Duration::from_secs(1),
  };

  let result = retry(example_operation, &retry_config).await;
  match result {
    Ok(output) => println!("Operation succeeded: {}", output),
    Err(err) => println!("Operation failed: {}", err),
  }
}
```


---
## 🚀 Contributing Guidelines

We welcome contributions to this project! Please follow these steps to contribute:

### 🐛 For Issues
- If you find an issue you'd like to work on, please comment on the issue and tag me (`@semicolon-10`) to assign it to you.  
  💡 *Tip*: Make sure the issue is not already assigned to someone else!
- Once assigned, you can start working on the issue. 🎉

### 🌟 For Planned Features
- If you'd like to work on a feature listed in the "Planned" section of the README, first create a new issue for that feature.  
  📝 *Note*: Clearly describe your approach or any details about how you plan to implement the feature.
- Tag me (`@semicolon-10`) in the issue and request assignment. 🙋‍♂️

### 🔧 Submitting Your Work
1. 🍴 Fork the repository and create a new branch for your work.
2. 🛠️ Make your changes and ensure they are well-tested.
3. ✅ Make sure all pipelines pass successfully before tagging me for review.
4. 📤 Submit a pull request (PR) with a clear description of the changes you made.
5. 🔗 Link the issue you worked on in the PR description.

### 🤝 Code of Conduct
- Be respectful and collaborative when interacting with other contributors. 🤗
- Ensure your code follows the project's coding standards and guidelines. ✅


### 🛠️ Example Workflow
1. 🔍 Find an issue or planned feature you'd like to work on.
2. 💬 Comment on the issue or create a new issue for the planned feature.
3. 🙋 Tag me (`@semicolon-10`) to assign the issue to you.
4. 🖊️ Work on the issue in your forked repository and submit a pull request.
---