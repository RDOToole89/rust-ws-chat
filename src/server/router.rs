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
        MessageType::ChatMessage {
            sender,
            receiver,
            content,
        } => {
            handle_chat_message(sender, receiver, content, &broadcaster).await?;
        }
        MessageType::SystemMessage {
            message_type,
            content,
        } => {
            handle_system_message(message_type, content, &broadcaster).await?;
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
        MessageType::Command(command) => {
            println!("[INFO]: Received command: {}", command);
        }
    }

    Ok(())
}
