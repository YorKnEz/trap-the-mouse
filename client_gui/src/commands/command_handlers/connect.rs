use std::net::SocketAddr;

use network::{request, Type};

use crate::{commands::CommandError, types::UserId, SERVER_ADDR};

pub fn connect_cmd(name: String, addr: SocketAddr) -> Result<UserId, CommandError> {
    let res: u32 = request(SERVER_ADDR, Type::Connect, &(name.clone(), addr))?;

    println!("received: {res}");

    Ok(res)
}
