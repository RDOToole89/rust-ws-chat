use futures_util::{SinkExt, StreamExt}; // Provides utilities for streams and sinks, used for WebSocket communication
use rust_ws_chat::message::message_type::{parse_message, MessageType};
use tokio::io::{self, AsyncBufReadExt, BufReader}; // Used for reading user input from stdin
use tokio_tungstenite::connect_async; // Asynchronous WebSocket connection function // Import MessageType enum and parse_message function for JSON parsing

#[tokio::main] // Marks the main function as an async function using the Tokio runtime
async fn main() {
    // The WebSocket server address
    let url = "ws://127.0.0.1:8080";

    // Connect to the WebSocket server asynchronously
    let (socket, _) = connect_async(url).await.expect("Failed to connect");
    println!("Connected to server");

    // Split the WebSocket connection into a sender and receiver
    // - `socket_sender` is used to send messages to the server.
    // - `socket_receiver` is used to receive messages from the server.
    let (mut socket_sender, mut socket_receiver) = socket.split();

    // Spawn a background task to handle incoming messages
    tokio::spawn(async move {
        // Process each incoming message from the WebSocket stream
        while let Some(msg) = socket_receiver.next().await {
            match msg {
                // Handle a Text message (JSON string) received from the server
                Ok(tokio_tungstenite::tungstenite::Message::Text(raw_message)) => {
                    // Attempt to parse the raw JSON string into the MessageType enum
                    match parse_message(&raw_message) {
                        Ok(message_type) => match message_type {
                            // If it's a SystemMessage, print the message
                            MessageType::SystemMessage(message) => println!("{}", message),

                            // If it's a ChatMessage, display the sender and content
                            MessageType::ChatMessage { sender, content } => {
                                println!("{}: {}", sender, content);
                            }

                            // If it's a UserList message, print the list of users
                            MessageType::UserList(users) => println!("Users: {:?}", users),

                            // Placeholder for unknown message types (in case you add more in the future)
                            _ => println!("Unknown message type"),
                        },

                        // Handle JSON parsing errors
                        Err(e) => println!("Failed to parse message: {:?}", e),
                    }
                }

                // Ignore other WebSocket message types (e.g., binary, ping, pong, close)
                Ok(_) => {}

                // Handle errors while receiving messages
                Err(e) => println!("Error: {:?}", e),
            }
        }
    });

    // Create a buffered reader for user input from stdin
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    println!("Type a message to send to the server (Ctrl+C to exit)");

    // Main loop to read user input and send messages to the server
    while let Ok(Some(line)) = lines.next_line().await {
        // Attempt to send the user's input as a message to the server
        if let Err(e) = socket_sender.send(line.into()).await {
            eprintln!("Failed to send message: {}", e);
            break; // Exit the loop if sending fails
        }
    }

    // This will only execute if the loop breaks (e.g., due to an error or user exiting)
    println!("Client disconnected from server");
}
