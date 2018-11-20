use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;

pub struct Connection(pub Arc<Mutex<Inner>>);

pub struct Inner {
    tcp_stream: TcpStream,
    busy: bool,
}

impl Connection {
    pub fn new(tcp_stream: TcpStream) -> Self {
        Connection(Arc::new(Mutex::new(Inner {
            tcp_stream,
            busy: false,
        })))
    }

    pub fn available(&self) -> bool {
        let guard = self.0.lock().unwrap();
        !guard.busy
    }

    pub fn handle(&self) -> TcpStream {
        let guard = self.0.lock().unwrap();
        guard.tcp_stream.try_clone().unwrap()
    }

    pub fn park(&self) {
        let mut guard = self.0.lock().unwrap();
        guard.busy = false;
    }

    pub fn unpark(&self) {
        let mut guard = self.0.lock().unwrap();
        guard.busy = true;
    }
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        Connection(self.0.clone())
    }
}
