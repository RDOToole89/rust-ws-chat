use crate::server::user_manager::UserManager;
use tokio::sync::broadcast::Sender;

// Handler for incoming messages
pub async fn handle_chat_message(
    sender: String,
    content: String,
    broadcaster: &Sender<String>,
) -> Result<(), String> {
    let message = format!("[{}] {}", sender, content);
    broadcaster.send(message).map_err(|e| e.to_string())?;
    Ok(())
}

// Handle system messages
pub async fn handle_system_message(
    message: String,
    broadcaster: &Sender<String>,
) -> Result<(), String> {
    let message = format!("[SYSTEM] {}", message);
    broadcaster.send(message).map_err(|e| e.to_string())?;
    Ok(())
}

// Handle user list requests
pub async fn handle_user_list_request(
    user_manager: &UserManager,
    broadcaster: &Sender<String>,
) -> Result<(), String> {
    let users = user_manager.list_users().await;
    let message = format!(
        "Current users: {}",
        users
            .iter()
            .map(|u| u.username.clone())
            .collect::<Vec<_>>()
            .join(", ")
    );
    broadcaster.send(message).map_err(|e| e.to_string())?;
    Ok(())
}
