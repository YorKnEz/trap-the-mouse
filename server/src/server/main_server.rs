use std::cell::RefCell;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use super::{LobbyServer, ServerT};
use network::{SendRecv, Type};

pub struct Server {
    server: Arc<Mutex<TcpListener>>,
    handles: RefCell<Vec<JoinHandle<()>>>,
    state: Arc<Mutex<ServerState>>,
}

pub struct ServerState {
    id: u64,
    lobbies: Vec<LobbyServer>,
}

impl ServerState {
    fn new() -> ServerState {
        ServerState {
            id: 0,
            lobbies: Vec::new(),
        }
    }
}

impl Server {
    pub fn new(addr: &str) -> Server {
        Server {
            server: Arc::new(Mutex::new(TcpListener::bind(addr).unwrap())),
            handles: RefCell::new(vec![]),
            state: Arc::new(Mutex::new(ServerState::new())),
        }
    }
}

impl ServerT for Server {
    const THREAD_POOL_SIZE: u32 = 2;

    fn start(&self) -> () {
        let mut handles = self.handles.borrow_mut();

        for _ in 0..Server::THREAD_POOL_SIZE {
            let server = Arc::clone(&self.server);
            let state = Arc::clone(&self.state);

            let handle = thread::spawn(move || loop {
                println!("{:?} waiting for connection", thread::current().id());

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

        // let mut state = self.state.lock()?;
    }
}

fn handle_connection(
    _state: &Arc<Mutex<ServerState>>,
    client: &mut TcpStream,
    addr: &SocketAddr,
) -> Result<()> {
    println!("new client: {addr:?}");

    let (buf, req_type) = match client.recv() {
        Ok(res) => res,
        Err(e) => return Err(anyhow!(format!("couldn't send: {e:?}"))),
    };

    let (res_type, res) = match req_type {
        Type::Ping => execute(buf, req_ping)?,
        Type::CreateLobby => execute_stateful(buf, req_create_lobby, _state)?,
        Type::DeleteLobby => execute_stateful(buf, req_delete_lobby, _state)?,
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

fn execute_stateful<R, S>(
    buf: Vec<u8>,
    handler: fn(R, &mut ServerState) -> Result<S>,
    _state: &Arc<Mutex<ServerState>>,
) -> Result<(Type, Vec<u8>)>
where
    R: for<'a> Deserialize<'a>,
    S: Serialize,
{
    let mut state = match _state.lock() {
        Ok(state) => state,
        Err(e) => {
            return Err(anyhow!(format!("couldn't acquire state lock: {e:?}")));
        }
    };

    match handler(bincode::deserialize(&buf)?, &mut state) {
        Ok(res) => Ok((Type::Success, bincode::serialize(&res)?)),
        Err(e) => {
            println!("couldn't ping: {e:?}");
            Ok((Type::Error, bincode::serialize("internal error")?))
        }
    }
}

fn req_ping(str: String) -> Result<String> {
    println!("received: {str}");
    Ok(str)
}

fn req_create_lobby(_: Vec<u8>, state: &mut ServerState) -> Result<SocketAddr> {
    let lobby = LobbyServer::new("127.0.0.1:0", state.id);

    lobby.start();

    let addr = match lobby.server.lock() {
        Ok(server) => server.local_addr().unwrap(),
        Err(e) => return Err(anyhow!(format!("couldn't acquire lobby state lock: {e:?}"))),
    };

    state.lobbies.push(lobby);
    state.id += 1;

    Ok(addr)
}

fn req_delete_lobby(_: u32, _: &mut ServerState) -> Result<()> {
    Ok(())
}
