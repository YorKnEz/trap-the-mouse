use std::net::SocketAddr;

use network::{request, Type};

use crate::{
    commands::{Command, CommandError},
    UserId, SERVER_ADDR,
};

pub struct ConnectCmd {
    name: String,
    addr: SocketAddr,
    user_id: UserId,
}

impl ConnectCmd {
    pub fn new(name: String, addr: SocketAddr, user_id: UserId) -> ConnectCmd {
        ConnectCmd {
            name,
            addr,
            user_id,
        }
    }
}

impl Command for ConnectCmd {
    fn execute(&mut self) -> Result<(), CommandError> {
        let user_id: u32 = request(SERVER_ADDR, Type::Connect, &(self.name.clone(), self.addr))?;

        *self.user_id.borrow_mut() = user_id;

        println!("received: {user_id}");

        Ok(())
    }
}
