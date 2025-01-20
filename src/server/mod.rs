pub mod connection;
pub mod router;
pub mod user_manager;

use crate::server::connection::handle_connection;
use crate::server::user_manager::UserManager;
use std::error::Error;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

// Start the WebSocket server
pub async fn start(addr: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create a TCP listener for the server
    let listener = TcpListener::bind(addr).await?;

    // Create a user manager for managing connected users
    let user_manager = UserManager::new();

    // Create a broadcaster for broadcasting messages to all connected clients
    let (broadcaster, _) = broadcast::channel(100);

    while let Ok((stream, address)) = listener.accept().await {
        let peer_address = address.to_string();
        let user_manager = user_manager.clone();
        let broadcaster = broadcaster.clone();

        // Spawn a new task to handle the connection
        // This is necessary because the accept() method is blocking and we need to handle multiple connections concurrently
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, peer_address, user_manager, broadcaster).await
            {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }

    Ok(())
}
