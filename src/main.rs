use crate::error::Result;
use server::Server;
use tracing::info;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

pub mod connection;
mod error;
pub mod packet;
mod server;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_line_number(true)
                .with_file(true),
        )
        .init();

    info!("Starting Minecraft server");
    let mut server = Server::new().await?;

    server.run().await
}
