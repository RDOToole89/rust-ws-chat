use serde::{Deserialize, Serialize};

// Emum for different types of messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    SystemMessage(String),
    ChatMessage { sender: String, content: String },
    UserList(Vec<String>),
    Command(String),
}

// Helper to parse a raw JSON string into a MessageType
pub fn parse_message(raw: &str) -> Result<MessageType, serde_json::Error> {
    serde_json::from_str(raw)
}

// Helper to convert a 'MessageType' into a JSON string
pub fn to_json(msg: &MessageType) -> Result<String, serde_json::Error> {
    serde_json::to_string(msg)
}
