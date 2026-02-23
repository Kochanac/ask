mod tool;
mod hook;

use rig::client::{CompletionClient, ProviderClient};
use rig::providers;
use rig::agent;
use rig::streaming::StreamingPrompt;
use std::env;
use tool::{Bash, ReadFile};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Create OpenAI client
    let client = providers::openrouter::Client::from_env();

    let args: String = env::args()
        .skip(1)
        .collect::<Vec<String>>()
        .join(" ");
    println!("Prompt: {}", args);

    // Create agent with a single context prompt
    let comedian_agent = client
        .agent("qwen/qwen3-coder-next")
        .tool(ReadFile)
        .tool(Bash)
        .preamble("You are a comedian here to entertain the user using humour and jokes.")
        .default_max_turns(20)
        .build();

    let session_id = "main";
    let hook = hook::hook::SessionIdHook { session_id };

    // Prompt the agent and print the response
    let mut stream = comedian_agent
        .stream_prompt(args)
        .with_hook(hook)
        .await;

    agent::stream_to_stdout(&mut stream).await?;

    Ok(())
}