use fast_tx_processor::config::Config;
use fast_tx_processor::db::create_pool;
use fast_tx_processor::app::create_router;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config
    let config = Config::from_env()?;

    // Connect to database
    let pool = create_pool(&config.database_url).await?;

    // Run migrations (in a real app, use a proper migration tool)
    let migrator = sqlx::migrate::Migrator::new(std::path::Path::new("./migrations")).await?;
    migrator.run(&pool).await?;

    // Create router
    let app = create_router(pool);

    // Bind address
    let addr = SocketAddr::from((config.server_addr.parse::<std::net::IpAddr>()?, config.server_port));

    // Start server
    println!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
