use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::{accept_async, tungstenite::Message};

// User structure for managing connected users
#[derive(Debug, Clone)]
struct User {
    username: String,
    peer_address: String,
}

// In-memory user manager for managing connected users
#[derive(Clone)]
struct UserManager {
    users: Arc<Mutex<HashMap<String, User>>>,
}

// UserManager implementation
impl UserManager {
    fn new() -> Self {
        UserManager {
            // Initialize an empty hashmap for storing users
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Add a new user to the manager
    async fn add_user(&self, username: String, peer_address: String) {
        let user = User {
            username: username.clone(),
            peer_address: peer_address.clone(),
        };
        self.users.lock().await.insert(username.clone(), user);
    }

    // Remove a user from the manager
    async fn remove_user(&self, peer_address: &str) {
        self.users.lock().await.remove(peer_address); // Remove the user from the user manager
    }

    // List all connected users
    async fn list_users(&self) -> Vec<User> {
        // Lock the users map and collect all usernames
        self.users
            .lock() // Lock the users map
            .await // Wait for the lock to be acquired
            .values() // Get the values of the users map
            .cloned() // Clone the User values instead of just their usernames
            .collect() // Collect the usernames into a vector
    }
}

// Handle a single WebSocket connection
async fn handle_connection(
    stream: tokio::net::TcpStream, // The raw TCP stream representing the connection between the client and the server
    peer_address: String,          // String identifying the client's address
    user_manager: UserManager,     // UserManager instance for managing connected users
    broadcaster: broadcast::Sender<String>, // Sender for broadcasting messages to all connected clients
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Upgrade the TCP stream to a WebSocket connection
    // This is necessary because the WebSocket protocol is built on top of TCP
    // accept_async is a function from tokio_tungstenite that takes a TCP stream and returns a WebSocket stream
    let ws_stream = accept_async(stream)
        .await // await will block the current task until the WebSocket connection is established
        .expect("Failed to upgrade to WebSocket connection"); // If the connection fails, the function will panic with an error message

    // Split the WebSocket stream into a writer and a reader
    // This is necessary because the WebSocket stream is a duplex stream, meaning it can both send and receive messages
    let (mut write, mut read) = ws_stream.split();

    // Send a welcome message to the client
    write
        .send(Message::text("Enter your username:".to_string()))
        .await?; // Convert the error type

    // Read the username from the client
    let username = if let Some(Ok(Message::Text(name))) = read.next().await {
        name.trim().to_string() // Trim any whitespace from the username and convert it to a string
    } else {
        return Ok(()); // If the client does not send a username, return early
    };

    // Add the user to the user manager
    // We use clone because we need to use the peer_address and username later
    // Clone is used to create a deep copy of the peer_address and username
    user_manager
        .add_user(peer_address.clone(), username.clone()) // Add the user to the user manager we use clone because we need to use the peer_address and username later
        .await;

    // Broadcast the join message to all connected clients
    let join_message = format!("*** {} has joined the chat ***", username);
    broadcaster.send(join_message.clone()).unwrap_or_default(); // Unwrap_or_default is used to handle the case where the send operation fails

    // Handle reading and writing messages
    let read_task = {
        let broadcaster = broadcaster.clone(); // Clone the broadcaster to use it in the async block
        let username = username.clone(); // Clone the username to use it in the async block

        // This async block will run in a separate thread and will handle reading messages from the client
        // The async move keyword is used to create an async block that captures the broadcaster and username variables by value
        async move {
            // This loop will run indefinitely until the client disconnects
            while let Some(msg) = read.next().await {
                // Read.next() pauses the current task until a message is received
                if let Ok(Message::Text(msg)) = msg {
                    // If the message is a text message
                    let message = format!("[{}] {}", username, msg); // Format the message with the username and the message
                    if let Err(e) = broadcaster.send(message.clone()) {
                        return Err("Failed to send message to broadcaster: {}".into());
                    }
                }
            }
            Ok(()) as Result<(), Box<dyn Error + Send + Sync>>
        }
    };

    // This async block will run in a separate thread and will handle writing messages to the client
    let write_task = {
        let mut receiver = broadcaster.subscribe(); // Subscribe to the broadcaster to receive messages
                                                    // This loop will run indefinitely until the client disconnects
        async move {
            // This loop will run indefinitely until the client disconnects
            while let Ok(msg) = receiver.recv().await {
                // receiver.recv() pauses the current task until a message is receive
                write.send(Message::text(msg)).await.unwrap_or_default(); // Send the message to the client
                return Err("Failed to send message to client".into());
            }
            Ok(()) as Result<(), Box<dyn Error + Send + Sync>>
        }
    };

    // Try to join the read and write tasks
    // This will block the current task until both tasks are complete
    match tokio::try_join!(read_task, write_task) {
        Ok(_) => {
            eprintln!("Connection closed for user: {}", username);
        }
        Err(e) => {
            eprintln!("Connection error for user {}: {}", username, e);
        }
    }

    // Remove the user from the user manager
    user_manager.remove_user(&peer_address).await;

    // Broadcast the leave message to all connected clients
    let leave_message = format!("*** {} has left the chat ***", username);
    broadcaster.send(leave_message.clone()).unwrap_or_default();
    Ok(())
}

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
