mod events;

use std::net::SocketAddr;

use network::{request, Type};

use crate::events::EventLoop;

const SERVER_ADDR: &str = "127.0.0.1:20000";

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut buf = String::new();
    let mut user_id: u32 = 0;
    let mut lobbies: Vec<(u16, SocketAddr)> = Vec::new();

    loop {
        buf.clear();
        std::io::stdin().read_line(&mut buf).unwrap();
        buf = buf.trim().to_string();

        if buf == "ping server" {
            let res: String = match request(SERVER_ADDR, Type::Ping, &"01234567") {
                Ok(res) => res,
                Err(e) => {
                    println!("couldn't request: {e:?}");
                    continue;
                }
            };

            println!("received: {res}");
        } else if buf == "connect" {
            user_id = match request(SERVER_ADDR, Type::Connect, &("yorknez", event_loop.addr)) {
                Ok(res) => res,
                Err(e) => {
                    println!("couldn't request: {e:?}");
                    continue;
                }
            };

            println!("received: {user_id}");
        } else if buf == "disconnect" {
            match request(SERVER_ADDR, Type::Disconnect, &user_id) {
                Ok(res) => res,
                Err(e) => {
                    println!("couldn't request: {e:?}");
                    continue;
                }
            };

            println!("disconnected");
        } else if buf.starts_with("ping lobby") {
            let index = buf.split(" ").nth(2).unwrap().parse::<usize>().unwrap();

            let res: String = match request(lobbies[index].1, Type::Ping, &"01234567") {
                Ok(res) => res,
                Err(e) => {
                    println!("couldn't request: {e:?}");
                    continue;
                }
            };

            println!("received: {res}");
        } else if buf == "create lobby" {
            let (id, addr): (u16, SocketAddr) =
                match request(SERVER_ADDR, Type::CreateLobby, &user_id) {
                    Ok(res) => res,
                    Err(e) => {
                        println!("couldn't request: {e:?}");
                        continue;
                    }
                };

            println!("received: {id:?} {addr:?}");

            lobbies.push((id, addr));
        } else if buf.starts_with("delete lobby") {
            let index = buf.split(" ").nth(2).unwrap().parse::<usize>().unwrap();

            match request(SERVER_ADDR, Type::DeleteLobby, &(user_id, lobbies[index].0)) {
                Ok(res) => res,
                Err(e) => {
                    println!("couldn't request: {e:?}");
                    continue;
                }
            };

            lobbies.remove(index);

            println!("deleted lobby")
        } else if buf == "list lobbies" {
            for (i, lobby) in lobbies.iter().enumerate() {
                println!("{i:2}. {lobby:?}");
            }
        } else if buf.starts_with("get lobbies") {
            let start = buf.split(" ").nth(2).unwrap().parse::<u32>().unwrap();
            let offset = buf.split(" ").nth(3).unwrap().parse::<u32>().unwrap();

            let mut new_lobbies: Vec<(u16, SocketAddr)> =
                match request(SERVER_ADDR, Type::GetLobbies, &(user_id, start, offset)) {
                    Ok(res) => res,
                    Err(e) => {
                        println!("couldn't request: {e:?}");
                        continue;
                    }
                };

            println!("received: {new_lobbies:?}");

            lobbies.append(&mut new_lobbies);
            lobbies.sort_by_key(|a| a.0);
            lobbies.dedup_by(|a, b| a.0 == b.0);
        } else if buf.starts_with("join lobby") {
            let index = buf.split(" ").nth(2).unwrap().parse::<usize>().unwrap();

            match request(lobbies[index].1, Type::JoinLobby, &user_id) {
                Ok(res) => res,
                Err(e) => {
                    println!("couldn't request: {e:?}");
                    continue;
                }
            };
            println!("joined lobby")
        } else if buf.starts_with("leave lobby") {
            let index = buf.split(" ").nth(2).unwrap().parse::<usize>().unwrap();

            match request(lobbies[index].1, Type::LeaveLobby, &user_id) {
                Ok(res) => res,
                Err(e) => {
                    println!("couldn't request: {e:?}");
                    continue;
                }
            };

            println!("left lobby")
        } else if buf == "event" {
            if let Some(ev) = event_loop.get_event() {
                ev.execute().unwrap();
            }
        } else if buf == "clear" {
            let _ = std::process::Command::new("clear").status();
        } else if buf == "quit" {
            break;
        }
    }
}
