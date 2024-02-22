use restful_api::{config, run, setup_store};

/*
@desc Set-up environment variable and configuration, then start server.
 */
#[tokio::main]
async fn main() -> Result<(), handle_errors::Error> {
    dotenv::dotenv().ok();

    let config = config::Config::new().expect("Config can't be set");
    let store = setup_store(&config).await?;

    tracing::info!("Q&A server build id: {}", env!("restful-api-version"));
    run(config, store).await;

    Ok(())
}