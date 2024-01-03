use std::{
    io,
    net::{SocketAddr, TcpListener},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use anyhow::Result;

use network::{SendRecv, Type};

use crate::{
    types::{BoolMutex, EventQueue, EventQueueItem},
    ActiveLobby, LobbyVec,
};

use super::{LobbyClosingEvent, PlayerJoinedEvent, PlayerLeftEvent, BecameHostEvent};

pub struct EventLoop {
    running: BoolMutex,
    events: EventQueue,
    handle: Option<JoinHandle<()>>,
    pub addr: SocketAddr,
}

impl EventLoop {
    pub fn new(lobbies: LobbyVec, active_lobby: ActiveLobby) -> Result<EventLoop> {
        let running = Arc::new(Mutex::new(true));
        let events = Arc::new(Mutex::new(vec![]));

        let server = TcpListener::bind("127.0.0.1:0")?;
        server.set_nonblocking(true)?;

        let addr = server.local_addr()?;

        let handle = {
            let running = Arc::clone(&running);
            let events = Arc::clone(&events);

            thread::spawn(move || event_loop_thread(running, events, server, lobbies, active_lobby))
        };

        Ok(EventLoop {
            running,
            events,
            handle: Some(handle),
            addr,
        })
    }

    pub fn get_event(&self) -> Option<EventQueueItem> {
        let mut events = self.events.lock().unwrap();
        events.pop()
    }

    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        self.stop();

        // can unwrap here because constructor initializes handle so it always has a value
        match self.handle.take().unwrap().join() {
            Ok(_) => {}
            Err(e) => println!("thread panicked: {e:?}"),
        }
    }
}

pub fn event_loop_thread(
    running: BoolMutex,
    events: EventQueue,
    server: TcpListener,
    lobbies: LobbyVec,
    active_lobby: ActiveLobby,
) {
    loop {
        {
            let running = running.lock().unwrap();

            if !*running {
                // println!("stopping loop");
                break;
            }
        }

        let ev: Option<EventQueueItem> = match server.accept() {
            Ok((mut stream, _addr)) => {
                let (buf, req_type) = match stream.recv() {
                    Ok(res) => res,
                    Err(e) => {
                        println!("couldn't recv: {e:?}");
                        continue;
                    }
                };

                let ev: Option<EventQueueItem> = match req_type {
                    Type::PlayerJoined => match bincode::deserialize(&buf) {
                        Ok(buf) => Some(Box::new(PlayerJoinedEvent::new(
                            buf,
                            Arc::clone(&active_lobby),
                        ))),
                        Err(_) => None,
                    },
                    Type::PlayerLeft => match bincode::deserialize(&buf) {
                        Ok(buf) => Some(Box::new(PlayerLeftEvent::new(
                            buf,
                            Arc::clone(&active_lobby),
                        ))),
                        Err(_) => None,
                    },
                    Type::LobbyClosing => match bincode::deserialize(&buf) {
                        Ok(buf) => Some(Box::new(LobbyClosingEvent::new(
                            buf,
                            Arc::clone(&lobbies),
                            Arc::clone(&active_lobby),
                        ))),
                        Err(_) => None,
                    },
                    Type::BecameHost => match bincode::deserialize(&buf) {
                        Ok(buf) => Some(Box::new(BecameHostEvent::new(
                            buf,
                            Arc::clone(&active_lobby),
                        ))),
                        Err(_) => None,
                    },
                    _ => None,
                };

                if let Err(e) = stream.send(Type::Success, &[]) {
                    println!("couldn't send: {e:?}");
                }

                ev
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    continue;
                }

                println!("couldn't accept: {e:?}");
                continue;
            }
        };

        if let Some(ev) = ev {
            let mut events = events.lock().unwrap();
            events.push(ev);
        }
    }
}
