pub mod db;
mod lobby;
mod request_handlers;
mod server;
mod types;

pub use server::*;

use std::cell::RefCell;
use std::io;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::ops::Drop;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use anyhow::Result;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use request_handlers::ExitRequest;

use types::{RequestQueueItem, BoolMutex, RequestQueue, HandleVec};

const THREAD_POOL_SIZE: u32 = 2;

pub trait RequestHandler {
    fn handle(&self, stream: TcpStream) -> Result<RequestQueueItem>;
}

pub struct ServerCore {
    pub running: BoolMutex,
    pub db_pool: Pool<SqliteConnectionManager>,
    requests: RequestQueue,
    server: TcpListener,
    handles: HandleVec,
}

impl ServerCore {
    pub fn new(addr: &str) -> Result<ServerCore> {
        // bool used to indicate to all threads if server should stop
        let running = Arc::new(Mutex::new(true));

        // db connection pool
        let manager = SqliteConnectionManager::file("db.db");
        let db_pool = Pool::new(manager)?;

        // requests queue
        let requests = Arc::new((Mutex::new(vec![]), Condvar::new()));

        let server = TcpListener::bind(addr)?;
        server.set_nonblocking(true)?;

        // create threads that will take care of requests
        let mut handles = vec![];

        for _ in 0..THREAD_POOL_SIZE {
            let running = Arc::clone(&running);
            let requests = Arc::clone(&requests);

            let handle = thread::spawn(move || {
                let (lock, cond) = &*requests;

                loop {
                    {
                        let running = running.lock().unwrap();

                        if !*running {
                            // println!("stopping thread {:?}", thread::current().id());
                            break;
                        }
                    }

                    let mut req: RequestQueueItem = {
                        let mut requests = lock.lock().unwrap();

                        while requests.len() == 0 {
                            requests = cond.wait(requests).unwrap();
                        }

                        // surely no panic will happen because we get here only if there is something to pop from the queue
                        requests.pop().unwrap()
                    };

                    match req.execute() {
                        Ok(_) => {}
                        Err(e) => println!("error handling request: {e:?}"),
                    }
                }
            });
            handles.push(handle);
        }

        // return newly created server
        Ok(ServerCore {
            running,
            db_pool,
            requests,
            server,
            handles: RefCell::new(handles),
        })
    }

    pub fn start<T: RequestHandler>(&self, server: &T) -> Result<()> {
        for stream in self.server.incoming() {
            {
                let running = self.running.lock().unwrap();

                if !*running {
                    // println!("stopping server");
                    break;
                }
            }

            let stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        continue;
                    }

                    println!("couldn't accept: {e:?}");
                    continue;
                }
            };

            let req = match server.handle(stream) {
                Ok(req) => req,
                Err(e) => {
                    println!("couldn't handle request: {e:?}");
                    continue;
                }
            };

            self.push_request(req);
        }

        Ok(())
    }

    fn push_request(&self, request: RequestQueueItem) {
        let mut requests = self.requests.0.lock().unwrap();

        requests.push(request);

        self.requests.1.notify_one();
    }

    pub fn get_addr(&self) -> Result<SocketAddr> {
        Ok(self.server.local_addr()?)
    }
}

impl Drop for ServerCore {
    fn drop(&mut self) {
        let mut handles = self.handles.borrow_mut();

        // push an exit request for each thread
        for _ in 0..handles.len() {
            let req = Box::new(ExitRequest {});

            self.push_request(req);

            // println!("pushed exit request");
        }

        while let Some(handle) = handles.pop() {
            match handle.join() {
                Ok(_) => {}
                Err(e) => println!("thread panicked: {e:?}"),
            }
        }
    }
}
