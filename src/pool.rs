use failure;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use tokio::net::TcpStream;
use tokio::prelude::{future, Future, task::Task};

use crate::connection::Connection;
use crate::future_connection::FutureConnection;

const REDIS_URI: &str = "127.0.0.1:6379";

struct Inner {
    connections: Vec<Connection>,
    tasks: Mutex<VecDeque<Task>>,
}

pub struct Pool(Arc<Inner>);

impl Pool {
    pub fn connect(
        connection_count: usize,
    ) -> impl Future<Item = Pool, Error = failure::Error> {
        Self::build_connections(connection_count).map(|connections| {
            Pool::new(connections)
        })
    }

    pub fn get_connection(&self) -> impl Future<Item = Connection, Error = ()> {
        FutureConnection::new(self.clone())
    }

    fn build_connections(
        connection_count: usize,
    ) -> impl Future<Item = Vec<Connection>, Error = failure::Error> {
        let streams = (0..connection_count).fold(Vec::new(), |mut connections, _| {
            let connection = TcpStream::connect(&REDIS_URI.parse().unwrap()).map_err(Into::into);
            connections.push(connection);
            connections
        });

        future::join_all(streams).map(|streams| {
            streams
                .into_iter()
                .map(|stream| Connection::new(stream))
                .collect()
        })
    }

    fn new(connections: Vec<Connection>) -> Self {
        Pool(Arc::new(Inner {
            connections,
            tasks: Mutex::new(VecDeque::new())
        }))
    }

    pub fn enqueue(&self, task: Task) {
        self.0.tasks.lock().unwrap().push_back(task);
    }

    fn notify(&self) {
        if let Some(task) = self.0.tasks.lock().unwrap().pop_front() {
            task.notify();
        }
    }

    pub fn park(&self, connection: Connection) {
        connection.park();
        self.notify();
    }

    pub fn take_connection(&self) -> Option<Connection> {
        for connection in &self.0.connections {
            if connection.available() {
                connection.unpark();
                return Some(connection.clone());
            }
        }
        None
    }
}

impl Clone for Pool {
    fn clone(&self) -> Self {
        Pool(self.0.clone())
    }
}
