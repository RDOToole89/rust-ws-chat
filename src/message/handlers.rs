use crate::server::user_manager::UserManager;
use tokio::sync::broadcast::Sender;

use super::message_type::{parse_message, to_json, MessageType};
// Handles broadcasting a chat message to all connected clients
pub async fn handle_chat_message(
    sender: String,  // The sender of the message
    content: String, // The content of the message
    receiver: String,
    broadcaster: &Sender<String>, // The broadcast channel for sending messages
) -> Result<(), String> {
    let chat_message = MessageType::ChatMessage {
        sender,
        receiver: "all".to_string(),
        content,
    };
    let serialized = to_json(&chat_message).map_err(|e| e.to_string())?; // Serialize to JSON

    broadcaster.send(serialized).map_err(|e| e.to_string())?; // Broadcast the message and handle errors
    Ok(())
}

// Handler for broadcasting system messages
pub async fn handle_system_message(
    message_type: String,
    content: String,
    broadcaster: &Sender<String>,
) -> Result<(), String> {
    // Parse the message to check if it contains structured data
    if let Ok(parsed_message) = parse_message(&content) {
        match parsed_message {
            MessageType::ChatMessage {
                sender,
                receiver,
                content,
            } => {
                println!("[DEBUG]: handle_system_message {:?} {:?}", sender, content);
                // If it's a ChatMessage, format it nicely for display
                let formatted_message = format!("*** {} has joined the chat ***", sender);
                broadcaster
                    .send(formatted_message)
                    .map_err(|e| e.to_string())?;
            }
            MessageType::SystemMessage {
                message_type,
                content,
            } => {
                // If it's already a SystemMessage, broadcast it as-is
                broadcaster
                    .send(format!("[{}]: {:}", message_type, content))
                    .map_err(|e: tokio::sync::broadcast::error::SendError<String>| e.to_string())?;
            }
            _ => {
                // Handle other message types, if needed
                broadcaster
                    .send(format!("[SYSTEM]: Unhandled message type"))
                    .map_err(|e| e.to_string())?;
            }
        }
    } else {
        // If it's not JSON, treat it as plain text and send it
        broadcaster
            .send(format!("[{}]: {}", message_type, content))
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

// Handles user list requests by broadcasting the list of connected users
pub async fn handle_user_list_request(
    user_manager: &UserManager, // Reference to the UserManager for fetching user info
    broadcaster: &Sender<String>, // The broadcast channel for sending messages
) -> Result<(), String> {
    let users = user_manager.list_users().await; // Fetch the list of users from the UserManager
    let message = format!(
        "Current users: {}",
        users
            .iter()
            .map(|u| u.username.clone())
            .collect::<Vec<_>>()
            .join(", ") // Format the list of usernames
    );
    broadcaster.send(message).map_err(|e| e.to_string())?; // Broadcast the message and handle errors
    Ok(())
}
