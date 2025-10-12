---
name: Bug Report
about: Report a bug to help us improve
title: '[BUG] '
labels: bug
assignees: ''
---

## Bug Description

<!-- A clear and concise description of what the bug is -->

## To Reproduce

Steps to reproduce the behavior:

1.
2.
3.
4.

**Minimal Code Example:**

```rust
// Minimal reproducible example
use talk::Agent;

#[tokio::main]
async fn main() {
    // Your code here
}
```

## Expected Behavior

<!-- A clear and concise description of what you expected to happen -->

## Actual Behavior

<!-- What actually happened -->

## Error Message/Stack Trace

```
Paste error message or stack trace here
```

## Environment

- **Talk version**: <!-- e.g., 0.1.1 -->
- **Rust version**: <!-- Output of `rustc --version` -->
- **Operating System**: <!-- e.g., macOS 14.0, Ubuntu 22.04, Windows 11 -->
- **LLM Provider**: <!-- e.g., OpenAI, Anthropic -->

**Cargo.toml dependencies:**

```toml
[dependencies]
talk = "0.1.1"
tokio = { version = "1", features = ["full"] }
```

## Additional Context

<!-- Add any other context about the problem here -->

- Does this happen consistently or intermittently?
- Have you made any custom modifications?
- Are there any workarounds?

## Possible Solution

<!-- Optional: If you have suggestions on how to fix the bug -->

## Related Issues

<!-- Link any related issues here -->
