use futures_util::{SinkExt, StreamExt};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio_tungstenite::connect_async;

#[tokio::main]
async fn main() {
    let url = "ws://127.0.0.1:8080";
    let (socket, _) = connect_async(url).await.expect("Failed to connect");
    println!("Connected to server");

    // Split the socket once
    let (mut socket_sender, mut socket_receiver) = socket.split();

    tokio::spawn(async move {
        while let Some(msg) = socket_receiver.next().await {
            match msg {
                Ok(message) => println!("Received: {:?}", message),
                Err(e) => println!("Error: {:?}", e),
            }
        }
    });

    // Use BufReader to handle user input from stdin
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    println!("Type a message to send to the server (Ctrl+C to exit)");
    while let Ok(Some(line)) = lines.next_line().await {
        if let Err(e) = socket_sender.send(line.into()).await {
            eprintln!("Failed to send message: {}", e);
            break;
        }
    }

    println!("Client disconnected from server");
}
