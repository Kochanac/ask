use rig::agent::{HookAction, PromptHook, ToolCallHookAction};
use rig::completion::{CompletionModel, CompletionResponse, Message};
use rig::message::{AssistantContent, UserContent};

#[derive(Clone)]
pub struct SessionIdHook<'a> {
    pub session_id: &'a str,
}

impl<'a, M: CompletionModel> PromptHook<M> for SessionIdHook<'a> {
    async fn on_tool_call(
        &self,
        tool_name: &str,
        _tool_call_id: Option<String>,
        _internal_call_id: &str,
        args: &str,
    ) -> ToolCallHookAction {
        println!(
            "[{}] Tool {} with {}",
            self.session_id,
            tool_name,
            args,
        );
        ToolCallHookAction::Continue
    }

    async fn on_tool_result(
        &self,
        tool_name: &str,
        _tool_call_id: Option<String>,
        _internal_call_id: &str,
        _args: &str,
        _result: &str,
    ) -> HookAction {
        println!(
            "[{}] {} finished",
            self.session_id, tool_name
        );

        HookAction::cont()
    }

    async fn on_completion_call(&self, prompt: &Message, _history: &[Message]) -> HookAction {
        println!(
            "[{}] {}",
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
                "[{}] Received response: <non-serializable>",
                self.session_id
            );
        }

        HookAction::cont()
    }
}
