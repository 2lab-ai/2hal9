use genius_game_server::GeniusGameServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("genius_game_server=debug,tower_http=debug")
        .init();
    
    // Create and run server
    let server = GeniusGameServer::new();
    server.run("0.0.0.0:8080").await?;
    
    Ok(())
}