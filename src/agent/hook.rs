use rig::agent::{HookAction, PromptHook, ToolCallHookAction};
use rig::completion::{CompletionModel, CompletionResponse, Message};
use rig::message::{AssistantContent, UserContent};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::AgentEvent;

/// SessionIdHook for logging agent actions and capturing events
#[derive(Clone)]
pub struct SessionIdHook {
    /// Shared event buffer
    events: Arc<Mutex<VecDeque<AgentEvent>>>,
}

impl SessionIdHook {
    pub fn new(events: Arc<Mutex<VecDeque<AgentEvent>>>) -> Self {
        Self { events }
    }
}

impl<M: CompletionModel> PromptHook<M> for SessionIdHook {
    async fn on_tool_call(
        &self,
        tool_name: &str,
        tool_call_id: Option<String>,
        internal_call_id: &str,
        args: &str,
    ) -> ToolCallHookAction {
        let mut events = self.events.lock().unwrap();
        events.push_back(AgentEvent::ToolCall {
            tool_name: tool_name.to_string(),
            args: args.to_string(),
            tool_call_id,
            internal_call_id: internal_call_id.to_string(),
        });

        ToolCallHookAction::Continue
    }

    async fn on_tool_result(
        &self,
        tool_name: &str,
        tool_call_id: Option<String>,
        _internal_call_id: &str,
        _args: &str,
        result: &str,
    ) -> HookAction {
        let mut events = self.events.lock().unwrap();
        events.push_back(AgentEvent::ToolResult {
            tool_name: tool_name.to_string(),
            result: result.to_string(),
            tool_call_id,
        });

        HookAction::cont()
    }

    async fn on_completion_call(
        &self,
        prompt: &Message,
        _history: &[Message],
    ) -> HookAction {
        let mut events = self.events.lock().unwrap();
        // Convert prompt to text for event
        let text = match prompt {
            Message::User { content } => content
                .iter()
                .filter_map(|c| {
                    if let UserContent::Text(text_content) = c {
                        Some(text_content.text.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
            Message::Assistant { content, .. } => content
                .iter()
                .filter_map(|c| if let AssistantContent::Text(text_content) = c {
                    Some(text_content.text.clone())
                } else {
                    None
                })
                .collect::<Vec<_>>()
                .join("\n"),
        };

        // events.push_back(AgentEvent::UserMessage(text));

        HookAction::cont()
    }

    async fn on_completion_response(
        &self,
        _prompt: &Message,
        response: &CompletionResponse<M::Response>,
    ) -> HookAction {
        let mut events = self.events.lock().unwrap();
        // Extract text from the response - choice contains AssistantContent
        for content in response.choice.iter() {
            if let AssistantContent::Text(text_content) = content {
                events.push_back(AgentEvent::Text(text_content.text.clone()));
            }
        }

        HookAction::cont()
    }

    async fn on_text_delta(&self, text_delta: &str, _aggregated_text: &str) -> HookAction {
        let mut events = self.events.lock().unwrap();
        events.push_back(AgentEvent::Text(text_delta.to_string()));
        HookAction::cont()
    }

    async fn on_stream_completion_response_finish(
        &self,
        _prompt: &Message,
        _response: &M::StreamingResponse,
    ) -> HookAction {
        // Nothing special needed; all text is captured via on_text_delta.
        HookAction::cont()
    }
}
