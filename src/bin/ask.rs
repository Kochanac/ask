use ask::AskAgent;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Get command line args
    let args: String = std::env::args()
        .skip(1)
        .collect::<Vec<String>>()
        .join(" ");
    println!("Query: {}", args);

    // Initialize the agent
    let agent = AskAgent::init().await?;

    // Stream prompt
    agent.stream_prompt(args).await?;

    Ok(())
}
