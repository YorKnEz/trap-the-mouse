// TODO: optimize allocations for packet reading, it isnt necessary to allocate on each recv

thread_local! {
    static HEADER_BYTES: RefCell<[u8; 8]> = RefCell::new([0u8; 8]);
}

use std::cell::RefCell;
use std::io;
use std::net::TcpStream;
use std::{io::prelude::*, net::ToSocketAddrs};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
struct Header {
    size: u32,
    h_type: Type,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub enum Type {
    // value used for initializations
    Default,
    // main server requests
    Ping,
    Connect,
    Disconnect,
    FindLobby,
    CreateLobby,
    GetLobbies,
    ChangeName,
    // lobby requests
    GetLobbyState,
    JoinLobby,
    LeaveLobby,
    CloseLobby,
    MakeHost,
    BecomeRole,
    SendMessage,
    ChangedName,
    StartGame,
    MakeMove,
    // client notifications
    PlayerJoined,
    PlayerLeft,
    PlayerUpdated,
    GameStarted,
    GameUpdated,
    LobbyClosing,
    // responses
    Success,
    Error,
}

pub fn request<A, S, R>(addr: A, h_type: Type, req: &S) -> Result<R>
where
    A: ToSocketAddrs,
    S: Serialize,
    R: for<'a> Deserialize<'a>,
{
    let mut stream = match TcpStream::connect(addr) {
        Ok(c) => c,
        Err(e) => {
            if e.kind() == io::ErrorKind::ConnectionRefused {
                return Err(anyhow!(format!("server is offline")));
            }
            return Err(anyhow!(format!("couldn't connect: {e:?}")));
        }
    };

    let req = bincode::serialize(req)?;

    if let Err(e) = stream.send(h_type, &req) {
        return Err(anyhow!(format!("couldn't send: {e:?}")));
    }

    let (res, res_type) = match stream.recv() {
        Ok(r) => r,
        Err(e) => return Err(anyhow!(format!("couldn't recv: {e:?}"))),
    };

    if res_type == Type::Error {
        return Err(anyhow!(format!(
            "{}",
            bincode::deserialize::<String>(&res)?
        )));
    }

    Ok(bincode::deserialize(&res)?)
}

pub trait SendRecv {
    fn send(&mut self, h_type: Type, buf: &[u8]) -> Result<()>;
    fn recv(&mut self) -> Result<(Vec<u8>, Type)>;
}

impl SendRecv for TcpStream {
    fn send(&mut self, h_type: Type, buf: &[u8]) -> Result<()> {
        let h = Header {
            size: buf.len() as u32,
            h_type,
        };
        let h = bincode::serialize(&h)?;

        self.write_all(&h)?;
        self.write_all(buf)?;

        Ok(())
    }

    fn recv(&mut self) -> Result<(Vec<u8>, Type)> {
        let h = HEADER_BYTES.with(|buf| -> Result<Header> {
            {
                self.read_exact(&mut *buf.borrow_mut())?;
            }

            Ok(bincode::deserialize(&*buf.borrow())?)
        })?;

        let mut buf = vec![0u8; h.size as usize];
        self.read_exact(&mut buf)?;

        Ok((buf, h.h_type))
    }
}

#[cfg(test)]
mod tests {
    use std::net::{TcpListener, TcpStream};

    use super::*;

    #[test]
    fn request() {
        let _server = TcpListener::bind("127.0.0.1:20000").unwrap();
        let mut client = TcpStream::connect("127.0.0.1:20000").unwrap();
        let (mut server, _addr) = _server.accept().unwrap();

        let msg = "test message";
        client.send(Type::Ping, msg.as_bytes()).unwrap();

        let (buf, req_type) = server.recv().unwrap();
        assert_eq!(req_type, Type::Ping);

        server.send(Type::Success, &buf).unwrap();

        let (buf, _) = client.recv().unwrap();

        assert_eq!(String::from_utf8(buf).unwrap(), msg);
    }
}
