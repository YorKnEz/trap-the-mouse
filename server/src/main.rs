use core::*;

mod core;

fn main() {
    db::init_db().unwrap();

    let server = Server::new("127.0.0.1:20000").unwrap();

    server.start().unwrap();
}
