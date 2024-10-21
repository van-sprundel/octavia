use crate::connection::Connection;
use crate::error::Result;
use tokio::net::TcpListener;
use tracing::{error, info};

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub async fn new() -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:25565").await?;
        Ok(Self { listener })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Server listening on port 25565");

        loop {
            let (socket, addr) = self.listener.accept().await?;
            let mut connection = Connection::new(socket);
            info!(%addr, "New connection");

            tokio::spawn(async move {
                if let Err(e) = connection.handle_connection().await {
                    error!(%addr, error = %e, "Connection error");
                }
            });
        }
    }
}
