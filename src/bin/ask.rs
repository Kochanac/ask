use ask::ui::cli;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    cli::run().await
}
