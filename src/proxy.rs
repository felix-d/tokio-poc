use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::io::{copy};

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
        let (server_reader, server_writer) = connection.handle().split();
        let (client_reader, client_writer) = client_stream.split();

        let client_to_server = copy(client_reader, server_writer);
        let server_to_client = copy(server_reader, client_writer);

        client_to_server.select(server_to_client).map(|_| ()).map_err(|_| ())
    }
}
