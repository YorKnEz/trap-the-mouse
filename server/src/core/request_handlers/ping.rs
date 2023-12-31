use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::{SendRecv, Type};

use super::Request;

pub struct PingRequest {
    stream: TcpStream,
    str: String,
}

impl PingRequest {
    pub fn new(stream: TcpStream, str: String) -> PingRequest {
        PingRequest { stream, str }
    }

    fn handler(&self) -> Result<String> {
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
                (Type::Error, bincode::serialize("internal error")?)
            }
        };

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
