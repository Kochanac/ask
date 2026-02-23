use rig::agent::Agent;
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Message;
use rig::providers;
use rig::streaming::StreamingPrompt;
use rig::agent as rig_agent;
use crate::agent::hook::hook::SessionIdHook;

/// AskAgent wraps rig::agent::Agent with application-specific configuration
pub struct AskAgent {
    pub inner: Agent<providers::openrouter::CompletionModel, SessionIdHook<'static>>,
}

impl AskAgent {
    /// Initialize a new agent with the comedian personality and tools
    pub async fn init() -> Result<Self, anyhow::Error> {
        // Create OpenRouter client
        let client = providers::openrouter::Client::from_env();

        let session_id = "main";
        let hook = SessionIdHook { session_id };

        // Create agent with a single context prompt and hook
        let comedian_agent = client
            .agent("qwen/qwen3-coder-next")
            .hook(hook)
            .preamble("You are a comedian here to entertain the user using humour and jokes.")
            .default_max_turns(20)
            .build();

        // Return wrapped agent
        Ok(AskAgent {
            inner: comedian_agent,
        })
    }

    /// Stream a prompt and output to stdout
    pub async fn stream_prompt(&self, prompt: impl Into<Message> + Send) -> Result<(), anyhow::Error> {
        let mut stream = self.inner
            .stream_prompt(prompt)
            .await;

        rig_agent::stream_to_stdout(&mut stream).await?;

        Ok(())
    }
}
