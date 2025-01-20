use rust_ws_chat::server;
use server::start;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let addr = "127.0.0.1:8080";
    println!("Starting server on {}", addr);

    if let Err(e) = start(addr).await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}
