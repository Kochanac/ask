use ask::AskAgent;
use ask::AgentEvent;
use clap::Parser;
use std::io::{self, Write, stdout};
use std::sync::Arc;
use tokio::time::{self, Duration};

#[derive(Parser)]
struct Cli {
    /// Optional prompt for one-shot mode. If not provided, interactive REPL is started.
    prompt: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    // Initialize the agent
    let agent = Arc::new(AskAgent::init().await?);

    if let Some(prompt) = cli.prompt {
        // One-shot mode: send the prompt and print all events after completion
        println!("Thinking...");
        agent.send_user_message(prompt).await?;
        let events = agent.get_events();
        for event in events {
            print_event(&event);
        }
        println!(); // ensure newline at end
        return Ok(());
    }

    // REPL mode: spawn background poller that prints events as they arrive
    let agent_poller = Arc::clone(&agent);
    let poller_handle = tokio::spawn(async move {
        loop {
            let events = agent_poller.get_events();
            for event in events {
                print_event(&event);
            }
            time::sleep(Duration::from_millis(50)).await;
        }
    });

    // REPL loop for user input
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
        time::sleep(Duration::from_millis(50)).await;
    }

    // Cancel the poller task when REPL exits
    poller_handle.abort();

    Ok(())
}

/// Print a single agent event to stdout
fn print_event(event: &AgentEvent) {
    match event {
        AgentEvent::Text(text) => {
            if text.trim() == "" {
                return ()
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
        AgentEvent::UserMessage(_) => {} // already echoed locally
    }
}
