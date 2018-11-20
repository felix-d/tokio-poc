use tokio::prelude::{Async, Poll, Future, task};

use crate::connection::Connection;
use crate::pool::Pool;

pub struct FutureConnection {
    pool: Pool,
    enqueued: bool,
}

impl FutureConnection {
    pub fn new(pool: Pool) -> Self {
        FutureConnection {
            pool,
            enqueued: false,
        }
    }
}

impl Future for FutureConnection {
    type Item = Connection;
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.pool.take_connection() {
            Some(connection) => Ok(Async::Ready(connection.clone())),
            None => {
                if !self.enqueued {
                    self.enqueued = true;
                    self.pool.enqueue(task::current());
                }
                Ok(Async::NotReady)
            }
        }
    }
}
