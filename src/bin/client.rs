use futures_util::{SinkExt, StreamExt}; // Provides utilities for streams and sinks, used for WebSocket communication
use rust_ws_chat::message::message_type::{parse_message, to_json, MessageType};
use tokio::io::{self, AsyncBufReadExt, BufReader}; // Used for reading user input from stdin
use tokio_tungstenite::connect_async; // Asynchronous WebSocket connection function // Import MessageType enum and parse_message function for JSON parsing
use tokio_tungstenite::tungstenite::Message;

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
                    match parse_message(&raw_message) {
                        Ok(message_type) => match message_type {
                            // If it's a SystemMessage, print it as-is
                            MessageType::SystemMessage {
                                message_type,
                                content,
                            } => {
                                println!("[{}]: {}", message_type, content)
                            }

                            // If it's a ChatMessage, print the sender and content
                            MessageType::ChatMessage {
                                sender,
                                receiver,
                                content,
                            } => {
                                println!("[{}]: {}", sender, content);
                            }

                            // If it's a UserList, print the list of users
                            MessageType::UserList(users) => {
                                println!("[SYSTEM]: Connected users: {:?}", users);
                            }

                            // Handle unknown or unimplemented message types
                            _ => println!("[SYSTEM]: Unknown message type received"),
                        },
                        Err(e) => {
                            // Handle errors in JSON parsing
                            eprintln!("[ERROR]: Failed to parse message: {:?}", e);
                        }
                    }
                }

                // Ignore non-text WebSocket messages
                Ok(_) => {}

                // Handle WebSocket errors
                Err(e) => {
                    eprintln!("[ERROR]: WebSocket error: {:?}", e);
                }
            }
        }
    });

    // Create a buffered reader for user input from stdin
    // `BufReader` is used to wrap `stdin`, making it easier to read lines of text input from the user asynchronously.
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    println!("Type a message to send to the server (Ctrl+C to exit)");

    // Main loop to read user input and send messages to the server
    while let Ok(Some(line)) = lines.next_line().await {
        // Create a `ChatMessage` variant of the `MessageType` enum.
        // This represents a structured chat message that includes the sender's name and the message content.
        let message_type = MessageType::ChatMessage {
            sender: "client".to_string(), // Sender is hardcoded as "client" for now.
            receiver: "all".to_string(),
            content: line.clone(), // User's input is used as the message content.
        };

        // Serialize the `MessageType` enum instance into a JSON string.
        // This step ensures the data adheres to the agreed-upon structure for communication with the server.
        let serialized_message = to_json(&message_type).unwrap();

        // Wrap the serialized JSON string in a `Message::Text`, which is a WebSocket-compatible text message.
        // Send the message to the server using the WebSocket connection.
        if let Err(e) = socket_sender.send(Message::Text(serialized_message)).await {
            // If sending fails, print an error message and exit the loop.
            eprintln!("Failed to send message: {}", e);
            break;
        }
    }

    // This will only execute if the loop breaks (e.g., due to an error or user exiting).
    // It indicates the client has disconnected from the server.
    println!("Client disconnected from server");
}
