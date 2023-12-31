use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::{SendRecv, Type};

use super::Request;

pub struct InvalidRequest {
    stream: TcpStream,
}

impl InvalidRequest {
    pub fn new(stream: TcpStream) -> InvalidRequest {
        InvalidRequest { stream }
    }
}

impl Request for InvalidRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = (Type::Error, bincode::serialize("invalid request")?);

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
