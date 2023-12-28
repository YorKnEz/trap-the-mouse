mod server;

use server::{Server, ServerT};

fn main() {
    let server = Server::new("127.0.0.1:20000");

    server.start();

    server.drop();
}
