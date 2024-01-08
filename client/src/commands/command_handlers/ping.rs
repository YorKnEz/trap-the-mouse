use std::net::SocketAddr;

use network::{request, Type};

use crate::commands::CommandError;

pub fn ping_cmd(message: String, addr: SocketAddr) -> Result<(), CommandError> {
    let res: String = request(addr, Type::Ping, &message)?;

    println!("received: {res}");

    Ok(())
}
