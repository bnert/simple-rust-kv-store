use std::thread;
use std::env;

mod stream;
mod store;

use stream::stream::{DomainStream, TcpStream};
use store::store::KeyValueStore;

fn main() -> std::io::Result<()> {

    let args: Vec<String> = env::args().collect();
    let is_domain = args[1] == "domain";
    let is_help = args[1] == "help";

    if is_help {
        println!("Usage: kvs [tcp|domain] [addr]");
        return Ok(());
    }

    if args.len() < 3 {
        println!("Usage: kvs [tcp|domain] [addr]");
        return Ok(());
    }

    let (mut store, store_tx) = KeyValueStore::new();
    let addr = &args[2];

    if is_domain {
        let (mut socket, socket_tx) = DomainStream::new(&addr);
        thread::spawn(move || {
            socket.handle(store_tx);
        });
        store.handle(socket_tx);
    } else {
        let (mut tcp, tcp_tx) = TcpStream::new(&addr);
        thread::spawn(move || {
            tcp.handle(store_tx);
        });
        store.handle(tcp_tx);
    }
    Ok(())
}
