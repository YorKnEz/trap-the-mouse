use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::{SendRecv, Type};

use super::{Request, error::ServerError};

pub struct PingRequest {
    stream: TcpStream,
    str: String,
}

impl PingRequest {
    pub fn new(stream: TcpStream, str: String) -> PingRequest {
        PingRequest { stream, str }
    }

    fn handler(&self) -> Result<String, ServerError> {
        println!("received: {}", self.str);
        Ok(self.str.clone())
    }
}

impl Request for PingRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = match self.handler() {
            Ok(res) => (Type::Success, bincode::serialize(&res)?),
            Err(e) => {
                println!("couldn't ping: {e:?}");
                match e {
                    ServerError::Internal(_) => (Type::Error, bincode::serialize("internal error")?),
                    ServerError::API { message } => (Type::Error, bincode::serialize(&message)?)
                }
            }
        };

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
