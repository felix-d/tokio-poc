mod proxy;
mod connection;
mod pool;
mod future_connection;

use crate::proxy::Proxy;

fn main() {
    tokio::run(Proxy::connect());
}
