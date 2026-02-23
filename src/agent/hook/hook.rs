use rig::agent::{HookAction, PromptHook, ToolCallHookAction};
use rig::completion::{CompletionModel, CompletionResponse, Message};
use rig::message::{AssistantContent, UserContent};
use serde_json;

/// SessionIdHook for logging agent actions
#[derive(Clone)]
pub struct SessionIdHook<'a> {
    pub session_id: &'a str,
}

impl<'a, M: CompletionModel> PromptHook<M> for SessionIdHook<'a> {
    async fn on_tool_call(
        &self,
        tool_name: &str,
        tool_call_id: Option<String>,
        internal_call_id: &str,
        args: &str,
    ) -> ToolCallHookAction {
        println!(
            "[Session {}] Calling tool: {} with call ID: {tool_call_id} (internal: {internal_call_id}) with args: {}",
            self.session_id,
            tool_name,
            args,
            tool_call_id = tool_call_id.unwrap_or("<no call ID provided>".to_string()),
        );
        ToolCallHookAction::Continue
    }

    async fn on_tool_result(
        &self,
        tool_name: &str,
        _tool_call_id: Option<String>,
        _internal_call_id: &str,
        args: &str,
        result: &str,
    ) -> HookAction {
        println!(
            "[Session {}] Tool result for {} (args: {}): {}",
            self.session_id, tool_name, args, result
        );

        HookAction::cont()
    }

    async fn on_completion_call(&self, prompt: &Message, _history: &[Message]) -> HookAction {
        println!(
            "[Session {}] Sending prompt: {}",
            self.session_id,
            match prompt {
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
            }
        );

        HookAction::cont()
    }

    async fn on_completion_response(
        &self,
        _prompt: &Message,
        response: &CompletionResponse<M::Response>,
    ) -> HookAction {
        if let Ok(resp) = serde_json::to_string(&response.raw_response) {
            println!("[Session {}] Received response: {}", self.session_id, resp);
        } else {
            println!(
                "[Session {}] Received response: <non-serializable>",
                self.session_id
            );
        }

        HookAction::cont()
    }
}
