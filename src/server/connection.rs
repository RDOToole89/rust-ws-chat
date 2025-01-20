use crate::message::message_type::{parse_message, to_json, MessageType};
use crate::server::router::route_message;
use crate::server::user_manager::UserManager;
use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

pub async fn handle_connection(
    stream: tokio::net::TcpStream,
    peer_address: String,
    user_manager: UserManager,
    broadcaster: broadcast::Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Upgrade the TCP stream to a WebSocket stream
    let ws_stream = accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();

    // Prompt the user for their username
    write
        .send(Message::text(to_json(&MessageType::SystemMessage {
            message_type: "SYSTEM".to_string(),
            content: "Enter your username:".to_string(),
        })?))
        .await?;

    let username = if let Some(Ok(Message::Text(name))) = read.next().await {
        name.trim().to_string()
    } else {
        return Ok(());
    };

    // Wrap `username` in an `Arc` to share it safely across tasks
    let username = Arc::new(username);

    // Add the user to the user manager
    user_manager.add_user(&peer_address, &*username).await;

    // Broadcast a system message to all connected clients
    let join_message = format!("*** {} has joined the chat ***", username);
    let join_message_type = MessageType::SystemMessage {
        message_type: "SYSTEM".to_string(),
        content: join_message,
    };
    broadcaster
        .send(to_json(&join_message_type)?)
        .unwrap_or_default();

    println!("[DEBUG]: join_message_type {:?}", join_message_type);

    // Task to handle incoming messages from the client
    let read_task = {
        let broadcaster = broadcaster.clone();
        let username = username.clone();
        let user_manager = user_manager.clone();

        async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(raw_message)) => match parse_message(&raw_message) {
                        Ok(message_type) => {
                            if let Err(e) =
                                route_message(message_type, &username, &user_manager, &broadcaster)
                                    .await
                            {
                                eprintln!("Failed to route message from {}: {:?}", username, e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to parse message from {}: {}", username, e);
                        }
                    },
                    Ok(_) => {
                        // Handle non-text messages (e.g., pings/pongs) if necessary
                    }
                    Err(e) => {
                        eprintln!("Error reading message from {}: {:?}", username, e);
                        break;
                    }
                }
            }
            Ok::<(), Box<dyn Error + Send + Sync>>(())
        }
    };

    // Task to handle outgoing messages to the client
    let write_task = {
        let mut receiver = broadcaster.subscribe();
        let username_clone = username.clone();

        async move {
            while let Ok(msg) = receiver.recv().await {
                if let Err(e) = write.send(Message::text(msg)).await {
                    eprintln!("Failed to send message to {}: {:?}", username_clone, e);
                    break;
                }
            }
            Ok::<(), Box<dyn Error + Send + Sync>>(())
        }
    };

    // Join read and tasks and handle any errors
    let username_clone = username.clone(); // Clone the Arc for this block
    if let Err(e) = tokio::try_join!(read_task, write_task) {
        eprintln!("Error in connection: {} {:?}", username_clone, e);
    }

    user_manager.remove_user(&peer_address).await;
    let leave_message = format!("*** {} has left the chat ***", username);
    let leave_message_type = MessageType::SystemMessage {
        message_type: "SYSTEM".to_string(),
        content: leave_message,
    };

    // Serialize the message
    match to_json(&leave_message_type) {
        Ok(serialized_message) => {
            let mut retries = 3; // Number of retries
            while retries > 0 {
                // Retry loop
                if broadcaster.send(serialized_message.clone()).is_ok() {
                    // Send the message
                    break; // Exit loop on success
                }
                eprintln!("Retrying to broadcast leave message for {}...", username);
                retries -= 1;
            }

            if retries == 0 {
                eprintln!(
                    "Failed to broadcast leave message for {} after retries",
                    username
                );
            }
        }
        Err(e) => {
            eprintln!("Error serializing leave message for {}: {:?}", username, e);
        }
    }
    Ok(())
}
