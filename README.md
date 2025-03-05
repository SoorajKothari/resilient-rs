<h1 align="center">Resilient-rs</h1>
<div align="center">

<i>A Rust utility library for fault tolerance, including retry strategies, backoff mechanisms, failure handling and much more.</i>
<br>
<br>
<a href="https://discord.com/invite/BymX4aJeEQ"><img src="https://img.shields.io/discord/733027681184251937.svg?style=flat&label=Join%20Community&color=7289DA" alt="Join Community Badge"/></a>
<a href="https://github.com/semicolon-10/resilient-rs/graphs/contributors"><img alt="GitHub contributors" src="https://img.shields.io/github/contributors/semicolon-10/resilient-rs.svg"></a>
[![Crates.io](https://img.shields.io/crates/v/resilient-rs.svg)](https://crates.io/crates/resilient-rs)
[![Downloads](https://img.shields.io/crates/d/resilient-rs)](https://crates.io/crates/resilient-rs)
[![Docs.rs](https://docs.rs/resilient-rs/badge.svg)](https://docs.rs/resilient-rs/latest/resilient_rs/)
<br>
<br>
<i>💖 Loved the work? [Subscribe to my YouTube channel](https://www.youtube.com/@Semicolon10) or consider giving this repository a ⭐ to show your support!</i>
</div>


## Feature Overview

| **Feature**           | **Details**                        | **Status**      |
|-----------------------|------------------------------------|-----------------|
| **Retry**             | Basic retry functionality         | ✅ Stable       |
|                       | With Backoff (exponential)        | ✅ Stable       |
|                       | With Fallback                     | ✅ Stable     |
| **Circuit Breaker**   | Prevents cascading failures       | 🛠️ Planned     |
| **Logging**           | Comprehensive debugging support   | ✅ Stable       |
| **More Examples**     | Additional usage examples         | 🛠️ Planned     |

### Notes:
- **Supported Contexts**: All features are available for both synchronous and asynchronous operations.
---

## 📦 How to Use `resilient-rs`

Here’s a quick example of how to use the `resilient-rs` crate in your Rust project.

### 1️⃣ Add `resilient-rs` to Your `Cargo.toml`

Add the following line to your `Cargo.toml` file:

```toml
[dependencies]
resilient-rs = "0.4.0" # Replace with the latest version
```

OR

```bash
cargo add resilient-rs
```

#### Synchronous
```rust
use std::time::Duration;
use resilient_rs::config::RetryConfig;
use resilient_rs::synchronous::retry;

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
use reqwest::Client;
use resilient_rs::asynchronous::retry;
use resilient_rs::config::RetryConfig;

async fn fetch_url() -> Result<String, reqwest::Error> {
  let client = Client::new();
  let response = client.get("https://example.com")
          .send()
          .await?;

  if response.status().is_success() {
    response.text().await
  } else {
    Err(reqwest::Error::new(reqwest::StatusCode::from_u16(response.status().as_u16()).unwrap(), "Request failed"))
  }
}

#[tokio::main]
async fn main() {
  let retry_config = RetryConfig::default();

  let result = retry(fetch_url, &retry_config).await;
  match result {
    Ok(output) => println!("Operation succeeded: {}", output),
    Err(err) => println!("Operation failed: {}", err),
  }
}
```

---
## 🚀 Contributing Guidelines

We welcome your contributions! Here's how to get started:

### 🐛 Issues & 🌟 Features
- Find an issue or planned feature you'd like to work on.
- Comment on the issue (or create one for planned features) and tag me (`@semicolon-10`) for assignment.  
  💡 *Tip*: Ensure it's not already assigned!
- Once assigned, start working. 🎉

### 🔧 Submitting Work
1. 🍴 Fork the repo and create a new branch.
2. 🛠️ Make changes and test thoroughly.
3. ✅ Ensure git actions pass before tagging me for review.
4. 📤 Submit a PR with a clear description and link the issue.

### 🤝 Code of Conduct
- Be respectful and collaborative. 🤗
- Follow coding standards and guidelines. ✅
---