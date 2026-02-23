use ask::{AskAgent, AgentEvent};
use std::time::Duration;
use tokio;

/// Helper to get all events after sending a message
async fn get_events_after(agent: &AskAgent, prompt: &str) -> Vec<AgentEvent> {
    agent.send_user_message(prompt.to_string()).await.unwrap();
    // Small delay to ensure all events are flushed
    tokio::time::sleep(Duration::from_millis(100)).await;
    agent.get_events()
}

#[tokio::test]
async fn test_simple_text_response() {
    // Skip test if no API key
    if std::env::var("OPENROUTER_API_KEY").is_err() {
        eprintln!("Skipping test: OPENROUTER_API_KEY not set");
        return;
    }

    let agent = AskAgent::init().await.unwrap();

    let events = get_events_after(&agent, "Say hello in one word.").await;

    // Verify we got some text back
    let text_events: Vec<&AgentEvent> = events.iter().filter(|e| matches!(e, AgentEvent::Text(_))).collect();
    assert!(!text_events.is_empty(), "Expected at least one text event");

    // The last text event should contain some greeting
    if let AgentEvent::Text(text) = text_events.last().unwrap() {
        let greeting = text.to_lowercase();
        assert!(
            greeting.contains("hello") || greeting.contains("hi") || greeting.contains("hey"),
            "Expected a greeting, got: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_tool_call_emits_events() {
    // Skip test if no API key
    if std::env::var("OPENROUTER_API_KEY").is_err() {
        eprintln!("Skipping test: OPENROUTER_API_KEY not set");
        return;
    }

    let agent = AskAgent::init().await.unwrap();

    // Prompt that should trigger a bash command: list files
    let events = get_events_after(&agent, "List files in the current directory using the bash tool.").await;

    // Check for at least one ToolCall
    let tool_calls: Vec<&AgentEvent> = events.iter().filter(|e| matches!(e, AgentEvent::ToolCall { .. })).collect();
    assert!(!tool_calls.is_empty(), "Expected at least one tool call event");

    // Check for corresponding ToolResult
    let tool_results: Vec<&AgentEvent> = events.iter().filter(|e| matches!(e, AgentEvent::ToolResult { .. })).collect();
    assert!(!tool_results.is_empty(), "Expected at least one tool result event");

    // Verify the tool name is "bash"
    if let AgentEvent::ToolCall { tool_name, .. } = tool_calls[0] {
        assert_eq!(tool_name, "bash");
    }
}

#[tokio::test]
async fn test_user_message_event_included() {
    // Skip test if no API key
    if std::env::var("OPENROUTER_API_KEY").is_err() {
        eprintln!("Skipping test: OPENROUTER_API_KEY not set");
        return;
    }

    let agent = AskAgent::init().await.unwrap();

    let prompt = "What is the capital of France?";
    let events = get_events_after(&agent, prompt).await;

    // The first event should be the UserMessage
    let first_event = events.first().unwrap();
    match first_event {
        AgentEvent::UserMessage(text) => assert_eq!(text, prompt),
        _ => panic!("First event should be UserMessage"),
    }
}

#[tokio::test]
async fn test_multiple_text_chunks_aggregated() {
    // Skip test if no API key
    if std::env::var("OPENROUTER_API_KEY").is_err() {
        eprintln!("Skipping test: OPENROUTER_API_KEY not set");
        return;
    }

    let agent = AskAgent::init().await.unwrap();

    let events = get_events_after(&agent, "Tell me a short joke.").await;

    // Collect all text events
    let mut full_text = String::new();
    for event in &events {
        if let AgentEvent::Text(t) = event {
            full_text.push_str(t);
        }
    }

    // Ensure we got some text (multiple chunks may be combined)
    assert!(!full_text.is_empty(), "Expected some text content");
    // Optionally check for joke-like content
    let lower = full_text.to_lowercase();
    assert!(
        lower.contains("joke") || lower.contains("laugh") || lower.contains("funny") ||
        lower.contains("?") || lower.contains("!）"),
        "Response doesn't seem like a joke: {}",
        full_text
    );
}
