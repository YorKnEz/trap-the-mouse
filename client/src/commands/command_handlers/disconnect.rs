use std::{cell::RefCell, rc::Rc};

use network::{request, Type};

use crate::{commands::{Command, CommandError}, SERVER_ADDR, UserId};

pub struct DisconnectCmd {
    user_id: UserId,
}

impl DisconnectCmd {
    pub fn new(user_id: Rc<RefCell<u32>>) -> DisconnectCmd {
        DisconnectCmd {
            user_id,
        }
    }
}

impl Command for DisconnectCmd {
    fn execute(&mut self) -> Result<(), CommandError> {
        request(SERVER_ADDR, Type::Disconnect, &*self.user_id.borrow())?;

        println!("disconnected");

        Ok(())
    }
}
