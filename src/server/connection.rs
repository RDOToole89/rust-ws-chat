use crate::message::message_type::{parse_message, to_json, MessageType};
use crate::server::router::route_message;
use crate::server::user_manager::UserManager;
use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

pub async fn handle_connection(
    stream: tokio::net::TcpStream,
    peer_address: String,
    user_manager: UserManager,
    broadcaster: broadcast::Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let ws_stream = accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();

    write
        .send(Message::text(to_json(&MessageType::SystemMessage(
            "Enter your username:".to_string(),
        ))?))
        .await?;

    let username = if let Some(Ok(Message::Text(name))) = read.next().await {
        name.trim().to_string()
    } else {
        return Ok(());
    };

    user_manager
        .add_user(peer_address.clone(), username.clone())
        .await;

    let join_message = format!("*** {} has joined the chat ***", username);
    let join_message_type = MessageType::SystemMessage(join_message);
    broadcaster
        .send(to_json(&join_message_type)?)
        .unwrap_or_default();

    let read_task = {
        let broadcaster = broadcaster.clone();
        let username = username.clone();
        let user_manager = user_manager.clone();

        async move {
            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(raw_message)) = msg {
                    let message_type = parse_message(&raw_message)?;
                    route_message(message_type, &username, &user_manager, &broadcaster).await?;
                }
            }
            Ok::<(), Box<dyn Error + Send + Sync>>(())
        }
    };

    let write_task = {
        let mut receiver = broadcaster.subscribe();

        async move {
            while let Ok(msg) = receiver.recv().await {
                write.send(Message::text(msg)).await?;
            }
            Ok::<(), Box<dyn Error + Send + Sync>>(())
        }
    };

    tokio::try_join!(read_task, write_task).unwrap_or_default();

    user_manager.remove_user(&peer_address).await;

    let leave_message = format!("*** {} has left the chat ***", username);
    let leave_message_type = MessageType::SystemMessage(leave_message);
    broadcaster
        .send(to_json(&leave_message_type)?)
        .unwrap_or_default();

    Ok(())
}
