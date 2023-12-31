use std::net::SocketAddr;

use network::{request, Type};

fn main() {
    let res: String = match request("127.0.0.1:20000", Type::Ping, &"01234567") {
        Ok(res) => res,
        Err(e) => return println!("couldn't request: {e:?}"),
    };

    println!("received: {res}");

    let addr: SocketAddr = match request("127.0.0.1:20000", Type::CreateLobby, &"") {
        Ok(res) => res,
        Err(e) => return println!("couldn't request: {e:?}"),
    };

    println!("received: {addr:?}");

    let res: String = match request(addr, Type::Ping, &"01234567") {
        Ok(res) => res,
        Err(e) => return println!("couldn't request: {e:?}"),
    };

    println!("received: {res}");

    let res: String = match request(addr, Type::DeleteLobby, &(addr.port() as u16)) {
        Ok(res) => res,
        Err(e) => return println!("couldn't request: {e:?}"),
    };

    println!("received: {res:?}");
}
