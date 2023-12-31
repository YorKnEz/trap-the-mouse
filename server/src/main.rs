// use server::{Server, ServerT};
use core::*;

// mod server;
mod core;

fn main() {
    let server = Server::new("127.0.0.1:20000").unwrap();

    server.start().unwrap();
}
