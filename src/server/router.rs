use crate::message::handlers::{handle_chat_message, handle_system_message};
use crate::message::message_type::MessageType;
use crate::server::user_manager::UserManager;
use tokio::sync::broadcast;

pub async fn route_message(
    message: MessageType,
    _username: &str,
    user_manager: &UserManager,
    broadcaster: &broadcast::Sender<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match message {
        MessageType::ChatMessage { sender, content } => {
            handle_chat_message(sender, content, &broadcaster).await?;
        }
        MessageType::SystemMessage(system_message) => {
            handle_system_message(system_message, &broadcaster).await?;
        }
        MessageType::UserList(_) => {
            let users = user_manager
                .list_users()
                .await
                .into_iter()
                .map(|u| u.username)
                .collect::<Vec<String>>();

            let user_list_message = MessageType::UserList(users);
            let serialized = serde_json::to_string(&user_list_message)?;
            broadcaster.send(serialized).unwrap_or_default();
        }
        MessageType::Command(_) => {
            // Handle command or do nothing for now
        }
    }

    Ok(())
}
