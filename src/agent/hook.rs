use rig::agent::{HookAction, PromptHook, ToolCallHookAction};
use rig::completion::{CompletionModel, CompletionResponse, Message};
use rig::message::AssistantContent;
use tokio::sync::mpsc;

use crate::AgentEvent;

/// SessionIdHook for logging agent actions and capturing events
#[derive(Clone)]
pub struct SessionIdHook {
    /// Sender for events
    events_tx: mpsc::UnboundedSender<AgentEvent>,
}

impl SessionIdHook {
    pub fn new(events_tx: mpsc::UnboundedSender<AgentEvent>) -> Self {
        Self { events_tx }
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
        self.events_tx
            .send(AgentEvent::ToolCall {
                tool_name: tool_name.to_string(),
                args: args.to_string(),
                tool_call_id,
                internal_call_id: internal_call_id.to_string(),
            })
            .ok();
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
        self.events_tx
            .send(AgentEvent::ToolResult {
                tool_name: tool_name.to_string(),
                result: result.to_string(),
                tool_call_id,
            })
            .ok();
        HookAction::cont()
    }

    async fn on_completion_call(
        &self,
        _prompt: &Message,
        _history: &[Message],
    ) -> HookAction {
        // UserMessage is emitted by AskAgent::send_user_message.
        HookAction::cont()
    }

    async fn on_completion_response(
        &self,
        _prompt: &Message,
        response: &CompletionResponse<M::Response>,
    ) -> HookAction {
        for content in response.choice.iter() {
            if let AssistantContent::Text(text_content) = content {
                self.events_tx
                    .send(AgentEvent::Text(text_content.text.clone()))
                    .ok();
            }
        }
        HookAction::cont()
    }

    async fn on_text_delta(&self, text_delta: &str, _aggregated_text: &str) -> HookAction {
        self.events_tx
            .send(AgentEvent::Text(text_delta.to_string()))
            .ok();
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
