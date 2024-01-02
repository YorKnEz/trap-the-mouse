use std::net::TcpStream;

use anyhow::{anyhow, Result};
use network::SendRecv;

use super::{error::ServerError, error_check, Request};

pub struct InvalidRequest {
    stream: TcpStream,
    reason: String,
}

impl InvalidRequest {
    pub fn new(stream: TcpStream, reason: &str) -> InvalidRequest {
        InvalidRequest { stream, reason: reason.to_string() }
    }

    fn handler(&self) -> Result<String, ServerError> {
        Err(ServerError::API{ message: self.reason.clone() })
    }
}

impl Request for InvalidRequest {
    fn execute(&mut self) -> Result<()> {
        let (res_type, res) = error_check(self.handler())?;

        if let Err(e) = self.stream.send(res_type, &res) {
            return Err(anyhow!(format!("couldn't send: {e:?}")));
        }

        Ok(())
    }
}
