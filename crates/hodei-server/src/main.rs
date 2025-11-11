/// hodei-server main entry point
use hodei_server::init_tracing;
use hodei_server::load_config;
use hodei_server::HodeiServer;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = load_config()
        .map_err(|e| {
            eprintln!("Configuration error: {}", e);
            e
        })
        .unwrap_or_else(|_| {
            eprintln!("Failed to load configuration");
            std::process::exit(1);
        });

    // Initialize logging
    init_tracing(config.debug);

    info!("Starting hodei-server v{}", env!("CARGO_PKG_VERSION"));
    info!("Configuration: {:#?}", config);

    // Create and run server
    let server = HodeiServer::new(config)
        .await
        .map_err(|e| {
            eprintln!("Failed to create server: {}", e);
            e
        })
        .unwrap_or_else(|_| {
            eprintln!("Failed to initialize server");
            std::process::exit(1);
        });

    server.run().await.map_err(|e| {
        eprintln!("Server error: {}", e);
        e
    })?;

    Ok(())
}
