mod commands;
mod events;
mod types;

use std::{
    cell::RefCell,
    net::{Ipv4Addr, SocketAddr},
    rc::Rc,
    sync::{Arc, Mutex},
};

use events::EventLoop;

use commands::{
    ClearCmd, CloseLobbyCmd, CommandError, ConnectCmd, CreateLobbyCmd, DisconnectCmd,
    GetLobbiesCmd, JoinLobbyCmd, LeaveLobbyCmd, PingCmd,
};

use types::{ActiveLobby, CmdQueue, Lobby, LobbyVec, UserId};

const SERVER_ADDR: SocketAddr =
    SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 20000);

fn main() {
    let mut buf = String::new();
    let user_id: UserId = Rc::new(RefCell::new(0));
    let lobbies: LobbyVec = Arc::new(Mutex::new(vec![]));

    let active_lobby: ActiveLobby = Arc::new(Mutex::new((
        Lobby {
            id: 0,
            addr: SERVER_ADDR,
            players: vec![],
        },
        false,
    )));

    let event_loop = EventLoop::new(Arc::clone(&lobbies), Arc::clone(&active_lobby)).unwrap();

    let commands: CmdQueue = Rc::new(RefCell::new(vec![]));

    loop {
        buf.clear();
        std::io::stdin().read_line(&mut buf).unwrap();
        buf = buf.trim().to_string();

        if buf == "ping server" {
            commands
                .borrow_mut()
                .push(Box::new(PingCmd::new("01234567".to_string(), SERVER_ADDR)));
        } else if buf == "connect" {
            commands.borrow_mut().push(Box::new(ConnectCmd::new(
                "yorknez".to_string(),
                event_loop.addr,
                Rc::clone(&user_id),
            )));
        } else if buf == "disconnect" {
            commands
                .borrow_mut()
                .push(Box::new(DisconnectCmd::new(Rc::clone(&user_id))));
        } else if buf.starts_with("ping lobby") {
            let index = buf.split(" ").nth(2).unwrap().parse::<usize>().unwrap();

            let lobbies = lobbies.lock().unwrap();

            commands.borrow_mut().push(Box::new(PingCmd::new(
                "01234567".to_string(),
                lobbies[index].addr,
            )));
        } else if buf == "create lobby" {
            commands.borrow_mut().push(Box::new(CreateLobbyCmd::new(
                Rc::clone(&user_id),
                Arc::clone(&lobbies),
            )));
        } else if buf.starts_with("close lobby") {
            commands.borrow_mut().push(Box::new(CloseLobbyCmd::new(
                Rc::clone(&user_id),
                Arc::clone(&active_lobby),
                Arc::clone(&lobbies),
            )));
        } else if buf == "list lobbies" {
            let lobbies = lobbies.lock().unwrap();

            for (i, lobby) in lobbies.iter().enumerate() {
                println!("{i:2}. {lobby:?}");
            }
        } else if buf.starts_with("get lobbies") {
            let start = buf.split(" ").nth(2).unwrap().parse::<u32>().unwrap();
            let offset = buf.split(" ").nth(3).unwrap().parse::<u32>().unwrap();

            commands.borrow_mut().push(Box::new(GetLobbiesCmd::new(
                Rc::clone(&user_id),
                start,
                offset,
                Arc::clone(&lobbies),
            )));
        } else if buf.starts_with("join lobby") {
            let index = buf.split(" ").nth(2).unwrap().parse::<usize>().unwrap();

            let lobby = {
                let lobbies = lobbies.lock().unwrap();
                lobbies[index]
            };

            commands.borrow_mut().push(Box::new(JoinLobbyCmd::new(
                Rc::clone(&user_id),
                lobby,
                Arc::clone(&active_lobby),
            )));
        } else if buf == "leave lobby" {
            commands.borrow_mut().push(Box::new(LeaveLobbyCmd::new(
                Rc::clone(&user_id),
                Arc::clone(&active_lobby),
            )));
        } else if buf == "active lobby" {
            let active_lobby = active_lobby.lock().unwrap();

            if active_lobby.1 {
                println!("{}", active_lobby.0);
            } else {
                println!("no lobby joined");
            }
        } else if buf == "event" {
            if let Some(ev) = event_loop.get_event() {
                ev.execute().unwrap();
            }
        } else if buf == "clear" {
            commands.borrow_mut().push(Box::new(ClearCmd::new()));
        } else if buf == "quit" {
            break;
        }

        if let Some(mut cmd) = commands.borrow_mut().pop() {
            match cmd.execute() {
                Ok(_) => {}
                Err(CommandError::InternalAnyhow(e)) => println!("cmd error: {e}"),
                Err(CommandError::CommandError { message }) => println!("cmd error: {message}"),
                Err(e) => println!("cmd error: {}", e),
            }
        }
    }
}
