use network::{request, Type};

use crate::{
    commands::CommandError,
    types::{Lobby, UserId},
};

pub fn send_message_cmd(
    user_id: &UserId,
    text: String,
    active_lobby: &Option<Lobby>,
) -> Result<(), CommandError> {
    if active_lobby.is_none() {
        return Err(CommandError::NotConnected);
    }

    if text.is_empty() {
        return Err(CommandError::EmptyString);
    }

    request(
        active_lobby.as_ref().unwrap().addr,
        Type::SendMessage,
        &(*user_id, text),
    )?;

    println!("message sent");

    Ok(())
}
