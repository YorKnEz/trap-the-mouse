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
    if let None = active_lobby {
        return Err(CommandError::CommandError {
            message: "you are not connected to a lobby".to_string(),
        });
    }

    request(
        active_lobby.as_ref().unwrap().addr,
        Type::MakeHost,
        &(*user_id, new_host_id),
    )?;

    println!("user {} will be host", new_host_id);

    Ok(())
}
