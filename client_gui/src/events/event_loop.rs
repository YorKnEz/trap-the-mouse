use std::{
    io,
    net::{SocketAddr, TcpListener},
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

use anyhow::Result;

use network::{SendRecv, Type};

use crate::types::{BoolMutex, EventQueue, EventQueueItem};

use super::{
    Event, GameStartedEvent, GameUpdatedEvent, LobbyClosingEvent, NetworkEvent, PlayerJoinedEvent,
    PlayerLeftEvent, PlayerUpdatedEvent, UIEvent,
};

pub struct EventLoop {
    running: BoolMutex,
    events: EventQueue,
    handles: Vec<Option<JoinHandle<()>>>,
    pub addr: SocketAddr,
    pub sender: mpsc::Sender<UIEvent>,
}

impl EventLoop {
    pub fn new() -> Result<EventLoop> {
        let running = Arc::new(Mutex::new(true));
        let events = Arc::new(Mutex::new(vec![]));

        let server = TcpListener::bind("127.0.0.1:0")?;
        server.set_nonblocking(true)?;

        let addr = server.local_addr()?;

        let mut handles = vec![];

        handles.push(Some({
            let running = Arc::clone(&running);
            let events = Arc::clone(&events);

            thread::spawn(move || network_event_loop_thread(running, events, server))
        }));

        let (sender, receiver) = mpsc::channel::<UIEvent>();

        handles.push(Some({
            let running = Arc::clone(&running);
            let events = Arc::clone(&events);

            thread::spawn(move || ui_event_loop_thread(running, events, receiver))
        }));

        Ok(EventLoop {
            running,
            events,
            handles,
            addr,
            sender,
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
        for handle in self.handles.iter_mut() {
            match handle.take().unwrap().join() {
                Ok(_) => {}
                Err(e) => println!("thread panicked: {e:?}"),
            }
        }
    }
}

pub fn network_event_loop_thread(running: BoolMutex, events: EventQueue, server: TcpListener) {
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

                let ev: Option<NetworkEvent> = match req_type {
                    Type::PlayerJoined => match bincode::deserialize(&buf) {
                        Ok(buf) => Some(NetworkEvent::PlayerJoined(PlayerJoinedEvent::new(buf))),
                        Err(_) => None,
                    },
                    Type::PlayerLeft => match bincode::deserialize(&buf) {
                        Ok(buf) => Some(NetworkEvent::PlayerLeft(PlayerLeftEvent::new(buf))),
                        Err(_) => None,
                    },
                    Type::PlayerUpdated => match bincode::deserialize(&buf) {
                        Ok(buf) => Some(NetworkEvent::PlayerUpdated(PlayerUpdatedEvent::new(buf))),
                        Err(_) => None,
                    },
                    Type::GameStarted => match bincode::deserialize(&buf) {
                        Ok(buf) => Some(NetworkEvent::GameStarted(GameStartedEvent::new(buf))),
                        Err(_) => None,
                    },
                    Type::GameUpdated => match bincode::deserialize(&buf) {
                        Ok(buf) => Some(NetworkEvent::GameUpdated(GameUpdatedEvent::new(buf))),
                        Err(_) => None,
                    },
                    Type::LobbyClosing => match bincode::deserialize(&buf) {
                        Ok(buf) => Some(NetworkEvent::LobbyClosing(LobbyClosingEvent::new(buf))),
                        Err(_) => None,
                    },
                    _ => None,
                };

                if let Err(e) = stream.send(Type::Success, &[]) {
                    println!("couldn't send: {e:?}");
                }

                ev.map(Event::Network)
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

pub fn ui_event_loop_thread(
    running: BoolMutex,
    events: EventQueue,
    receiver: mpsc::Receiver<UIEvent>,
) {
    loop {
        {
            let running = running.lock().unwrap();

            if !*running {
                // println!("stopping loop");
                break;
            }
        }

        if let Ok(ev) = receiver.try_recv() {
            let mut events = events.lock().unwrap();
            events.push(Event::UI(ev));
        }
    }
}
