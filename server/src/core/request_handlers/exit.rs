// this request is launched by the server in order to shut down it's threads
// if it weren't for this request, all threads would be stuck on the conditional wait forever

// use std::thread;

use anyhow::Result;

use super::Request;

pub struct ExitRequest {}

impl Request for ExitRequest {
    fn execute(&mut self) -> Result<()> {
        // println!("exit received {:?}", thread::current().id());
        Ok(())
    }
}
