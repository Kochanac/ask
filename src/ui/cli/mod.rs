use crate::agent::AskAgent;
use crate::AgentEvent;
use clap::Parser;
use std::io::{self, Write, stdout};
use tokio::task;
use tokio::time::Duration;

#[derive(Parser, Debug)]
pub struct CliArgs {
    /// Optional initial prompt. All remaining arguments are joined with spaces.
    #[clap(allow_hyphen_values = true)]
    pub prompt: Vec<String>,
}

/// Main entry point for the CLI. Initializes the agent and runs the event loop.
pub async fn run() -> Result<(), anyhow::Error> {
    // Initialize the agent and get the event receiver
    let (agent, mut events_rx) = AskAgent::init().await?;

    // Spawn a background task that consumes events and prints them as they arrive
    let consumer_handle = tokio::spawn(async move {
        while let Some(event) = events_rx.recv().await {
            print_event(&event);
        }
    });

    // If an initial prompt was provided, send it first
    let cli = CliArgs::parse();
    if !cli.prompt.is_empty() {
        let initial_prompt = cli.prompt.join(" ");
        agent.send_user_message(initial_prompt).await?;
        // Allow a moment for events to flush before REPL begins
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    // REPL loop
    loop {
        print!("\n> ");
        let _ = stdout().flush();
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let line = line.trim().to_string();

        if line == "exit" || line == "quit" {
            println!("Goodbye!");
            break;
        }

        if line.is_empty() {
            continue;
        }

        if let Err(e) = agent.send_user_message(line).await {
            eprintln!("Error: {}", e);
        }

        // Yield to allow the consumer task to print events before next prompt
        task::yield_now().await;
    }

    // Clean shutdown: abort the consumer task
    consumer_handle.abort();

    Ok(())
}

/// Print a single agent event to stdout
fn print_event(event: &AgentEvent) {
    match event {
        AgentEvent::Text(text) => {
            if text.trim().is_empty() {
                return;
            }
            print!("{}", text);
            let _ = stdout().flush();
        }
        AgentEvent::ToolCall { tool_name, args, .. } => {
            println!("[Tool] {}: {}", tool_name, args.trim());
        }
        AgentEvent::ToolResult { tool_name, result: _, .. } => {
            println!("[Result] {}", tool_name);
        }
        AgentEvent::ApprovalRequest { tool_name, args, tool_call_id } => {
            println!("[Approval needed] {}: {} (id: {})", tool_name, args.trim(), tool_call_id);
        }
        AgentEvent::ApprovalApproved { tool_call_id } => {
            println!("[Approved] {}", tool_call_id);
        }
        AgentEvent::ApprovalDenied { tool_call_id } => {
            println!("[Denied] {}", tool_call_id);
        }
        AgentEvent::UserMessage(_) => {} // already echoed
    }
}
