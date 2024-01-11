mod commands;
mod events;
mod types;

use std::net::{Ipv4Addr, SocketAddr};
use std::{cell::RefCell, rc::Rc};

use commands::{
    become_role_cmd, change_name_cmd, check_error, clear_cmd, close_lobby_cmd, connect_cmd,
    create_lobby_cmd, disconnect_cmd, get_lobbies_cmd, get_lobby_state, join_lobby_cmd,
    leave_lobby_cmd, make_host_cmd, ping_cmd,
};
use events::EventLoop;
use types::{GameState, GameStateShared, LobbyShort, LobbyVec, UserType};

const SERVER_ADDR: SocketAddr =
    SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 20000);
const DEFAULT_NAME: &str = "Player";

fn main() {
    let mut buf = String::new();

    let game_state: GameStateShared = rc_cell!(GameState {
        id: 0, // invalid id, doesn't matter because we connect before using the id
        name: String::from(DEFAULT_NAME),
        lobby: None,
        selected_lobby: None,
    });

    let mut lobbies: LobbyVec = vec![];

    let event_loop = EventLoop::new().unwrap();

    loop {
        buf.clear();
        std::io::stdin().read_line(&mut buf).unwrap();
        buf = buf.trim().to_string();

        let mut state = game_state.borrow_mut();

        if buf == "ping" {
            match ping_cmd("01234567".to_string(), SERVER_ADDR) {
                Ok(_) => {}
                Err(e) => check_error(e),
            }
        } else if buf == "connect" {
            match connect_cmd(state.name.clone(), event_loop.addr) {
                Ok(id) => state.id = id,
                Err(e) => check_error(e),
            }
        } else if buf == "disconnect" {
            match disconnect_cmd(&state.id) {
                Ok(_) => {}
                Err(e) => check_error(e),
            }
        } else if buf.starts_with("change name") {
            let new_name = buf
                .chars()
                .enumerate()
                .filter(|(_, c)| *c == ' ')
                .map(|(i, _)| i)
                .nth(1)
                .unwrap();
            let new_name = String::from(&buf[new_name + 1..]);

            match change_name_cmd(&state.id, new_name.clone(), &state.lobby) {
                Ok(_) => state.name = new_name,
                Err(e) => check_error(e),
            }
        } else if buf.starts_with("ping lobby") {
            let index = buf.split(' ').nth(2).unwrap().parse::<usize>().unwrap();

            if index < lobbies.len() {
                match ping_cmd("01234567".to_string(), lobbies[index].addr) {
                    Ok(_) => {}
                    Err(e) => check_error(e),
                }
            }
        } else if buf.starts_with("create lobby") {
            let lobby_name = buf
                .chars()
                .enumerate()
                .filter(|(_, c)| *c == ' ')
                .map(|(i, _)| i)
                .nth(1)
                .unwrap();
            let lobby_name = String::from(&buf[lobby_name + 1..]);

            match create_lobby_cmd(&state.id, lobby_name) {
                Ok(lobby) => match get_lobby_state(lobby.clone()) {
                    Ok(lobby_state) => lobbies.push(LobbyShort {
                        id: lobby.id,
                        addr: lobby.addr,
                        name: lobby_state.name,
                        players: lobby_state.players,
                    }),
                    Err(e) => check_error(e),
                },
                Err(e) => check_error(e),
            }
        } else if buf == "list lobbies" {
            for (i, lobby) in lobbies.iter().enumerate() {
                println!("{i:2}. {lobby:?}");
            }
        } else if buf.starts_with("get lobbies") {
            let start = buf.split(' ').nth(2).unwrap().parse::<u32>().unwrap();
            let offset = buf.split(' ').nth(3).unwrap().parse::<u32>().unwrap();

            match get_lobbies_cmd(&state.id, start, offset) {
                Ok(new_lobbies) => {
                    for lobby in new_lobbies {
                        match get_lobby_state(lobby.clone()) {
                            Ok(lobby_state) => lobbies.push(LobbyShort {
                                id: lobby.id,
                                addr: lobby.addr,
                                name: lobby_state.name,
                                players: lobby_state.players,
                            }),
                            Err(e) => check_error(e),
                        }
                    }
                    lobbies.sort_by_key(|a| a.id);
                    lobbies.dedup_by_key(|a| a.id);
                }
                Err(e) => check_error(e),
            }
        } else if buf == "active lobby" {
            if let Some(active_lobby) = state.lobby.as_ref() {
                println!("{}", active_lobby);
            } else {
                println!("no lobby joined");
            }
        } else if buf.starts_with("join lobby") {
            let index = buf.split(' ').nth(2).unwrap().parse::<usize>().unwrap();

            if index < lobbies.len() {
                match join_lobby_cmd(&state.id, lobbies[index].addr, &state.lobby) {
                    Ok(lobby_state) => {
                        let mut user_type = UserType::Spectator;
                        for player in &lobby_state.players {
                            if player.id == state.id {
                                user_type = player.user_type;
                            }
                        }
                        state.lobby = Some(types::Lobby {
                            id: lobbies[index].id,
                            addr: lobbies[index].addr,
                            name: lobby_state.name,
                            players: lobby_state.players,
                            user_type,
                        });
                    }
                    Err(e) => check_error(e),
                }
            }
        } else if buf == "leave lobby" {
            match leave_lobby_cmd(&state.id.clone(), &mut state.lobby) {
                Ok(_) => {}
                Err(e) => check_error(e),
            }
        } else if buf == "close lobby" {
            match close_lobby_cmd(&state.id.clone(), &mut state.lobby) {
                Ok(id) => lobbies.retain(|a| a.id != id),
                Err(e) => check_error(e),
            }
        } else if buf.starts_with("make host") {
            let id = buf.split(' ').nth(2).unwrap().parse::<u32>().unwrap();

            match make_host_cmd(&state.id, id, &state.lobby) {
                Ok(_) => {}
                Err(e) => check_error(e),
            }
        } else if buf.starts_with("become") {
            let role = buf.split(' ').nth(1).unwrap();
            let role = if role == "player" {
                UserType::Player
            } else if role == "spectator" {
                UserType::Spectator
            } else {
                continue;
            };
            match become_role_cmd(&state.id, role, &state.lobby) {
                Ok(_) => {}
                Err(e) => check_error(e),
            }
        } else if buf == "clear" {
            match clear_cmd() {
                Ok(_) => {}
                Err(e) => check_error(e),
            }
        } else if buf == "quit" {
            break;
        }

        while let Some(e) = event_loop.get_event() {
            println!("{e:?}");
            match e {
                events::Event::Network(events::NetworkEvent::PlayerUpdated(e)) => {
                    if let Some(active_lobby) = state.lobby.as_mut() {
                        active_lobby.players.iter_mut().for_each(|p| {
                            if p.id == e.player.id {
                                *p = e.player.clone();
                            }
                        });
                    }
                }
                events::Event::Network(events::NetworkEvent::PlayerJoined(e)) => {
                    if let Some(active_lobby) = state.lobby.as_mut() {
                        active_lobby.players.push(e.player);
                    }
                }
                events::Event::Network(events::NetworkEvent::PlayerLeft(e)) => {
                    if let Some(active_lobby) = state.lobby.as_mut() {
                        active_lobby.players.retain(|p| p.id != e.user_id);
                    }
                }
                events::Event::Network(events::NetworkEvent::LobbyClosing(_)) => {
                    if let Some(active_lobby) = state.lobby.as_mut() {
                        lobbies.retain(|a| a.id != active_lobby.id);
                    }
                    state.lobby = None;
                }
                _ => {}
            }
        }
    }
}
