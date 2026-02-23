# Rig Quickstart

Documentation: [https://docs.rig.rs/docs/quickstart/getting_started](https://docs.rig.rs/docs/quickstart/getting_started)

## Overview

Rig is a Rust library for building LLM-powered applications with a focus on ergonomics and modularity.

## Key Features

- LLM completion and embedding workflows
- Unified interface for 20+ model providers (OpenAI, Anthropic, Cohere, etc.)
- Agent abstractions for complex LLM workflows
- Vector store integrations for RAG
- OpenTelemetry compatible (GenAI Semantic Conventions)

## Quick Start Example

```rust
use rig::{completion::Prompt, providers};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Create OpenAI client
    let client = providers::openai::Client::from_env();

    // Create agent
    let agent = client
        .agent("gpt-4o-mini")
        .preamble("You are a helpful assistant.")
        .build();

    // Prompt the agent
    let response = agent.prompt("Hello!").await?;
    println!("{}", response);

    Ok(())
}
```

## Setup

1. Add `rig-core` to your `Cargo.toml`:
```toml
[dependencies]
rig-core = "0.4"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

2. Set your API key:
```bash
export OPENAI_API_KEY="sk-..."
# or for OpenRouter:
export OPENROUTER_API_KEY="sk-or-..."
```

## Related Documentation

- [API Reference](https://docs.rs/rig-core/latest/rig/)
- [Examples](https://github.com/0xPlaygrounds/rig/tree/main/examples)

## Source Code

The Rig source code is located at: `/home/kochan/proj/ai/rig`
