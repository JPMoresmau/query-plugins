use anyhow::Result;
use query_runner_server::app;
use std::net::SocketAddr;

/// Web server entry point.
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app()?.into_make_service())
        .await?;
    Ok(())
}
