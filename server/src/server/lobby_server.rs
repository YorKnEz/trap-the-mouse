use std::cell::RefCell;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use super::ServerT;
use network::{SendRecv,Type};

pub struct LobbyServer {
    pub server: Arc<Mutex<TcpListener>>,
    handles: RefCell<Vec<JoinHandle<()>>>,
    pub state: Arc<Mutex<LobbyServerState>>,
}

pub struct LobbyServerState {
    _id: u64,
}

impl LobbyServerState {
    fn new(id: u64) -> LobbyServerState {
        LobbyServerState { _id: id }
    }
}

impl LobbyServer {
    pub fn new(addr: &str, id: u64) -> LobbyServer {
        LobbyServer {
            server: Arc::new(Mutex::new(TcpListener::bind(addr).unwrap())),
            handles: RefCell::new(vec![]),
            state: Arc::new(Mutex::new(LobbyServerState::new(id))),
        }
    }
}

impl ServerT for LobbyServer {
    const THREAD_POOL_SIZE: u32 = 2;

    fn start(&self) {
        let mut handles = self.handles.borrow_mut();

        for _ in 0..LobbyServer::THREAD_POOL_SIZE {
            let server = Arc::clone(&self.server);
            let state = Arc::clone(&self.state);

            let handle = thread::spawn(move || loop {
                println!("lobby {:?} waiting for connection", thread::current().id());

                let srv = match server.lock() {
                    Ok(srv) => srv,
                    Err(e) => {
                        println!("couldn't acquire lock: {e:?}");
                        continue;
                    }
                };

                let (mut client, addr) = match srv.accept() {
                    Ok((client, addr)) => (client, addr),
                    Err(e) => {
                        println!("couldn't get client: {e:?}");
                        continue;
                    }
                };

                if let Err(e) = handle_connection(&state, &mut client, &addr) {
                    println!("error during connection handling: {e:?}");
                    continue;
                }

                // connection can be further operated on here
            });

            handles.push(handle);
        }
    }

    fn drop(self) {
        for handle in self.handles.into_inner() {
            match handle.join() {
                Ok(_) => {}
                Err(e) => println!("couldn't join thread: {e:?}"),
            }
        }
    }
}

fn handle_connection(
    _state: &Arc<Mutex<LobbyServerState>>,
    client: &mut TcpStream,
    addr: &SocketAddr,
) -> Result<()> {
    println!("new client: {addr:?}");

    let (buf, req_type) = match client.recv() {
        Ok(res) => res,
        Err(e) => return Err(anyhow!(format!("couldn't send: {e:?}"))),
    };

    let (res_type, res) = match req_type {
        Type::Ping => { execute(buf, req_ping)? },
        _ => (Type::Error, bincode::serialize("invalid request")?),
    };

    if let Err(e) = client.send(res_type, &res) {
        return Err(anyhow!(format!("couldn't send: {e:?}")));
    }

    Ok(())
}

fn execute<R, S>(buf: Vec<u8>, handler: fn(R) -> Result<S>) -> Result<(Type, Vec<u8>)>
where
    R: for<'a> Deserialize<'a>,
    S: Serialize,
{
    match handler(bincode::deserialize(&buf)?) {
        Ok(res) => Ok((Type::Success, bincode::serialize(&res)?)),
        Err(e) => {
            println!("couldn't ping: {e:?}");
            Ok((Type::Error, bincode::serialize("internal error")?))
        }
    }
}

// fn execute_stateful<R, S>(buf: Vec<u8>, handler: fn(R, &mut LobbyServerState) -> Result<S>, _state: &Arc<Mutex<LobbyServerState>>) -> Result<(Type, Vec<u8>)>
// where
//     R: for<'a> Deserialize<'a>,
//     S: Serialize,
// {
//     let mut state = match _state.lock() {
//         Ok(state) => state,
//         Err(e) => {
//             return Err(anyhow!(format!("couldn't acquire state lock: {e:?}")));
//         }
//     };
//
//     match handler(bincode::deserialize(&buf)?, &mut state) {
//         Ok(res) => Ok((Type::Success, bincode::serialize(&res)?)),
//         Err(e) => {
//             println!("couldn't ping: {e:?}");
//             Ok((Type::Error, bincode::serialize("internal error")?))
//         }
//     }
// }

fn req_ping(str: String) -> Result<String> {
    println!("received: {str}");
    Ok(str)
}
