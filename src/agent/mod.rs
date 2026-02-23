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
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
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

/// AskAgent wraps rig::agent::Agent with application-specific configuration
/// and manages an event stream for decoupled UI interaction
pub struct AskAgent {
    pub inner: Agent<providers::openrouter::CompletionModel, SessionIdHook>,
    /// Shared event buffer for UI to consume (protected by Mutex)
    pub events: Arc<Mutex<VecDeque<AgentEvent>>>,
}

impl AskAgent {
    /// Initialize a new agent with the comedian personality and tools
    pub async fn init() -> Result<Self> {
        // Create OpenRouter client
        let client = providers::openrouter::Client::from_env();

        // Create shared event buffer
        let events = Arc::new(Mutex::new(VecDeque::new()));
        let hook = SessionIdHook::new(events.clone());

        // Create agent with a single context prompt
        let comedian_agent = client
            .agent("qwen/qwen3-coder-next")
            .hook(hook)
            .preamble("You are a helpful assistant.")
            .default_max_turns(20)
            .tool(crate::tool::Bash)
            .tool(crate::tool::ReadFile)
            .build();

        // Return wrapped agent
        Ok(AskAgent {
            inner: comedian_agent,
            events,
        })
    }

    /// Send a user message to the agent and process it asynchronously
    /// This will trigger the agent to respond and generate events
    pub async fn send_user_message(&self, message: String) -> Result<()> {
        let user_message = message.clone();
        // Clear previous events first, then record the new user message
        {
            let mut events = self.events.lock().unwrap();
            events.clear();
            events.push_back(AgentEvent::UserMessage(user_message.clone()));
        }

        // Convert to rig Message
        use rig::OneOrMany;
        let user_content = OneOrMany::one(UserContent::text(user_message));
        let rig_message = Message::User { content: user_content };

        // Stream the response with a total timeout to avoid hanging forever.
        let mut stream = self.inner.stream_prompt(rig_message).await;
        let stream_future = async {
            while let Some(chunk) = stream.next().await {
                // Drain the stream; hooks push events.
                // Convert any streaming error to anyhow::Error
                chunk.map_err(anyhow::Error::from)?;
            }
            Ok::<(), anyhow::Error>(())
        };
        // Timeout after 120 seconds; propagate errors
        timeout(Duration::from_secs(120), stream_future).await??;

        Ok(())
    }

    /// Get all pending events from the agent's event stream
    pub fn get_events(&self) -> Vec<AgentEvent> {
        let mut events = self.events.lock().unwrap();
        events.drain(..).collect()
    }

    /// Get the number of pending events
    pub fn event_count(&self) -> usize {
        self.events.lock().unwrap().len()
    }
}
