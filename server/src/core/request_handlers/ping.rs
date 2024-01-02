use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::SendRecv;

use super::{error::ServerError, error_check, Request};

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
        let (res_type, res) = error_check(self.handler())?;

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
