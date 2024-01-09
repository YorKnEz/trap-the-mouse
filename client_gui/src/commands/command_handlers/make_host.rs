use network::{request, Type};

use crate::{
    commands::CommandError,
    types::{Lobby, UserId},
};

pub fn make_host_cmd(
    user_id: &UserId,
    new_host_id: u32,
    active_lobby: &Option<Lobby>,
) -> Result<(), CommandError> {
    if active_lobby.is_none() {
        return Err(CommandError::NotConnected);
    }

    request(
        active_lobby.as_ref().unwrap().addr,
        Type::MakeHost,
        &(*user_id, new_host_id),
    )?;

    println!("user {} will be host", new_host_id);

    Ok(())
}
