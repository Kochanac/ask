use ask::AskAgent;
use ask::AgentEvent;
use clap::Parser;
use std::io::{self, Write, stdout};
use tokio::task;
use tokio::time::{self, Duration};

#[derive(Parser)]
struct Cli {
    /// Optional prompt for one-shot mode. If not provided, interactive REPL is started.
    prompt: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    // Initialize the agent and get the event receiver
    let (agent, mut events_rx) = AskAgent::init().await?;

    // Spawn a background task that consumes events and prints them as they arrive
    let consumer_handle = tokio::spawn(async move {
        while let Some(event) = events_rx.recv().await {
            print_event(&event);
        }
    });

    if let Some(prompt) = cli.prompt {
        // One-shot mode
        println!("Thinking...");
        agent.send_user_message(prompt).await?;

        // Give some time for any final events to flush and be printed
        time::sleep(Duration::from_millis(200)).await;

        // Abort the consumer task (it will be blocked on recv, but we're exiting)
        consumer_handle.abort();
        println!(); // ensure newline at end
        return Ok(());
    }

    // REPL mode
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

        // Give the consumer task a chance to print any pending events before we show the next prompt.
        // This mimics the behavior of the original poller-based implementation and ensures
        // events from the current turn appear before the next prompt.
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
