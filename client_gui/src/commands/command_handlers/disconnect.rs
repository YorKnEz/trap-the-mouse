use network::{request, Type};

use crate::{commands::CommandError, types::UserId, SERVER_ADDR};

pub fn disconnect_cmd(user_id: &UserId) -> Result<(), CommandError> {
    request(SERVER_ADDR.with(|&a| a), Type::Disconnect, user_id)?;

    println!("disconnected");

    Ok(())
}
