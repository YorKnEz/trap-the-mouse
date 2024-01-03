use std::net::SocketAddr;

use network::{request, Type};

use crate::commands::{Command, CommandError};

pub struct PingCmd {
    message: String,
    addr: SocketAddr,
}

impl PingCmd {
    pub fn new(message: String, addr: SocketAddr) -> PingCmd {
        PingCmd { message, addr }
    }
}

impl Command for PingCmd {
    fn execute(&mut self) -> Result<(), CommandError> {
        let res: String = request(self.addr, Type::Ping, &self.message)?;

        println!("received: {res}");

        Ok(())
    }
}
