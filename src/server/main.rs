mod server;
use std::error::Error; // Import the Error trait

// This is the main function that will start the server
// The #[tokio::main] attribute is used to mark the main function as an async function
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // The Result type is used to handle errors and the Box type is used to handle errors that are not known at compile time
    // Initialize logging
    env_logger::init();

    // Start the WebSocket server
    let addr = "127.0.0.1:8080";

    println!("Starting server on {}", addr);

    // start the server
    if let Err(e) = server::start(addr).await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}
