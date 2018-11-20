use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::timer::Delay;
use tokio::io::write_all;

use crate::connection::Connection;
use crate::pool::Pool;

const PORT: &str = "12345";
const CONNECTION_COUNT: usize = 1;

pub struct Proxy {
    pool: Pool,
}

impl Proxy {
    pub fn connect() -> impl Future<Item = (), Error = ()> {
        Pool::connect(CONNECTION_COUNT)
            .map_err(|_| ())
            .and_then(|pool| Proxy { pool }.listen())
    }

    fn listen(self) -> impl Future<Item = (), Error = ()> {
        let addr = format!("127.0.0.1:{}", PORT);
        println!("Proxy running on {}.", addr);
        let addr = addr.parse().unwrap();

        let listener = TcpListener::bind(&addr).unwrap();

        listener.incoming().map_err(|_| ()).for_each(move |client_stream| {
            let pool = self.pool.clone();
            let request = self.pool.get_connection().and_then(|connection| {
                Self::proxy(client_stream, connection.clone()).then(move |_| {
                    pool.park(connection);
                    Ok(())
                })
            });

            tokio::spawn(request)
        })
    }

    fn proxy(client_stream: TcpStream, connection: Connection) -> impl Future<Item = (), Error = ()> {
        // This is where the proxying happens. We have access to both client and server async tcp
        // streams now we need to send data from the client reader to the server writer and vice
        // versa. For some ideas on how this can be achieved see
        // https://github.com/tokio-rs/tokio/blob/master/examples/proxy.rs
        // However note that in this example, a new server connection is created for the duration
        // of the request. In our case we want to keep the server connection opened so it's
        // a bit more delicate.
        let _server_stream = connection.handle();

        // In the meantime, here's a POC that fakes a server response after a delay.
        let when = Instant::now() + Duration::from_millis(2000);

        Delay::new(when)
            .map_err(|_| ())
            .and_then(move |_| {
                write_all(client_stream, b"hello\n")
                    .map(|_| ())
                    .map_err(|_| ())
            })
    }
}
