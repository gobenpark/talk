# Talk

**A Rust library for creating controlled LLM agents with behavioral guidelines, tool integration, and multi-step conversation journeys.**

Talk enables developers to create production-ready AI agents with predictable behavior in under 50 lines of Rust code, featuring pluggable LLM providers (OpenAI, Anthropic), configurable session storage backends, and comprehensive explainability features.

## Features

- 🎯 **Behavioral Guidelines**: Define predictable agent behavior with pattern matching and priority-based execution
- 🔧 **Tool Integration**: Register async functions as tools with configurable timeouts
- 🗺️ **Conversation Journeys**: Multi-step conversation state machines for guided user flows
- 🔌 **Pluggable LLM Providers**: Built-in support for OpenAI and Anthropic with trait-based extensibility
- 💾 **Session Storage**: In-memory default with optional Redis and PostgreSQL backends
- 📊 **Explainability**: Understand agent decisions with comprehensive decision tracking
- ⚡ **Performance**: <2s response times, 1000+ concurrent sessions support
- 🦀 **Type-Safe**: Full Rust type safety with compile-time guarantees

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
talk = "0.1"
tokio = { version = "1", features = ["full"] }
```

### Simple Agent with Guidelines (30 lines)

```rust
use talk::{Agent, Guideline, GuidelineCondition, GuidelineAction, OpenAiProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create LLM provider
    let provider = OpenAiProvider::new(std::env::var("OPENAI_API_KEY")?);

    // Create agent
    let mut agent = Agent::builder()
        .name("Customer Support")
        .provider(provider)
        .build()?;

    // Define behavioral guideline
    let pricing_guideline = Guideline {
        condition: GuidelineCondition::Literal("pricing".to_string()),
        action: GuidelineAction {
            response_template: "Our pricing starts at $49/month for the basic plan.".to_string(),
            requires_llm: false,
            parameters: vec![],
        },
        priority: 10,
        ..Default::default()
    };

    // Register guideline and process message
    agent.add_guideline(pricing_guideline).await?;
    let session_id = agent.create_session().await?;
    let response = agent.process_message(session_id, "What is your pricing?".to_string()).await?;

    println!("Agent: {}", response.message);
    Ok(())
}
```

## Installation

### Prerequisites

- Rust 1.90 or later
- OpenAI or Anthropic API key
- Basic familiarity with async Rust

### Basic Installation

```bash
cargo add talk tokio --features tokio/full
```

### With Optional Storage Backends

```bash
# Redis storage
cargo add talk --features redis-storage

# PostgreSQL storage
cargo add talk --features postgres-storage

# All storage backends
cargo add talk --features all-storage
```

## Documentation

- [Quick Start Guide](specs/001-python-parlant-agent/quickstart.md)
- [Data Model](specs/001-python-parlant-agent/data-model.md)
- [API Contracts](specs/001-python-parlant-agent/contracts/api-contracts.md)
- [Full Specification](specs/001-python-parlant-agent/spec.md)

## Examples

See the `examples/` directory for more complex use cases:

- `simple_agent.rs` - Basic agent with guidelines
- `weather_agent.rs` - Agent with tool integration
- `onboarding.rs` - Multi-step journey example

## Performance

- Agent response time: <2s (excluding LLM latency)
- Tool integration overhead: <100ms
- Guideline matching: O(n) linear time with SIMD acceleration
- Concurrent sessions: 1000+ without degradation

## Architecture

Talk is built on:

- **Tokio**: Async runtime for high-performance concurrent operations
- **serde/serde_json**: Zero-cost serialization with type safety
- **async-openai & anthropic-sdk**: Official LLM provider integrations
- **Aho-Corasick + regex**: Efficient pattern matching for guidelines
- **thiserror**: Type-safe error handling

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

Inspired by [Parlant](https://github.com/emcie-co/parlant), the Python library for creating LLM-based agents.

---

**Ready to build production-ready AI agents in Rust!** 🦀
