pub mod hook;

use self::hook::SessionIdHook;
use anyhow::Result;
use futures::stream::StreamExt;
use rig::agent::Agent;
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Message;
use rig::message::UserContent;
use rig::providers;
use rig::streaming::StreamingPrompt;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

/// Events that can be emitted by the agent
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// New text from the model
    Text(String),
    /// Tool calls being made
    ToolCall {
        tool_name: String,
        args: String,
        tool_call_id: Option<String>,
        internal_call_id: String,
    },
    /// Tool call results
    ToolResult {
        tool_name: String,
        result: String,
        tool_call_id: Option<String>,
    },
    /// User message (from UI to agent)
    UserMessage(String),
    /// Request for approval of a tool call
    ApprovalRequest {
        tool_name: String,
        args: String,
        tool_call_id: String,
    },
    /// Tool call approved
    ApprovalApproved {
        tool_call_id: String,
    },
    /// Tool call denied
    ApprovalDenied {
        tool_call_id: String,
    },
}

/// AskAgent wraps rig::agent::Agent with application-specific configuration.
/// It emits events to an async channel. The caller receives the receiver side
/// to consume events.
pub struct AskAgent {
    pub inner: Agent<providers::openrouter::CompletionModel, SessionIdHook>,
    /// Sender for events (not exposed publicly; used by hook)
    events_tx: mpsc::UnboundedSender<AgentEvent>,
}

impl AskAgent {
    /// Initialize a new agent with the comedian personality and tools.
    /// Returns the agent and an unbounded receiver for events.
    pub async fn init() -> Result<(Self, mpsc::UnboundedReceiver<AgentEvent>)> {
        // Create OpenRouter client
        let client = providers::openrouter::Client::from_env();

        // Create unbounded channel for events
        let (events_tx, events_rx) = mpsc::unbounded_channel();
        let hook = SessionIdHook::new(events_tx.clone());

        let agent = client
            .agent("qwen/qwen3-coder-next")
            .hook(hook)
            .preamble("You are a helpful agent. Use tools that are provided to you to complete the user's task. Never ask user to do something, that you can do yourself.")
            .default_max_turns(100)
            .tool(crate::tool::Bash)
            .tool(crate::tool::ReadFile)
            .build();

        // Return wrapped agent and the receiver
        Ok((
            AskAgent {
                inner: agent,
                events_tx,
            },
            events_rx,
        ))
    }

    /// Send a user message to the agent and process it asynchronously.
    /// Emits events to the event channel, including the user message itself.
    pub async fn send_user_message(&self, message: String) -> Result<()> {
        // Emit user message event
        self.events_tx
            .send(AgentEvent::UserMessage(message.clone()))
            .ok();

        // Convert to rig Message
        use rig::OneOrMany;
        let user_content = OneOrMany::one(UserContent::text(message));
        let rig_message = Message::User { content: user_content };

        // Stream the response with a total timeout to avoid hanging forever.
        let mut stream = self.inner.stream_prompt(rig_message).await;
        let stream_future = async {
            while let Some(chunk) = stream.next().await {
                chunk.map_err(anyhow::Error::from)?;
            }
            Ok::<(), anyhow::Error>(())
        };
        // Timeout after 120 seconds; propagate errors
        timeout(Duration::from_secs(120), stream_future).await??;

        Ok(())
    }
}
