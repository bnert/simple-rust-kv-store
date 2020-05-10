pub mod stream {
    use std::fs;
    use std::sync::mpsc;
    use std::io::{Read, Write};
    use std::os::unix::net::UnixListener;
    use std::net::TcpListener;

    pub struct DomainStream
    {
        stream_buff: String,
        listener: UnixListener,
        rx: mpsc::Receiver<String>,
    }

    impl DomainStream {
        pub fn new(path: &str) -> (DomainStream, mpsc::Sender<String>) {
           match fs::remove_file(path) {
                Ok(_) => println!("OK"),
                _ => println!("INIT"),
           }

           println!("Domain fd: {}", path);
            let (tx, rx) = mpsc::channel();
            (DomainStream{
                stream_buff: String::new(),
                listener: UnixListener::bind(path).unwrap(),
                rx: rx
            }, mpsc::Sender::clone(&tx))
        }

        pub fn handle(&mut self, sender: mpsc::Sender<String>) {
            for stream in self.listener.incoming() {
                match stream {
                    Ok(mut s) => {
                        let res = s.read_to_string(&mut self.stream_buff);
                        match res {
                            Ok(_) => {
                                sender.send(self.stream_buff.clone()).unwrap();
                                self.stream_buff.clear();
                            }
                            _ => println!("Err")
                        }

                        let to_send = self.rx.recv().unwrap().into_bytes();
                        s.write(&to_send).unwrap();
                    }
                    _ => println!("ERR"),
                }
            }
        }
    }

    pub struct TcpStream {
        stream_buff: String,
        listener: TcpListener,
        rx: mpsc::Receiver<String>
    }

    impl TcpStream {
        pub fn new(addr: &str) -> (TcpStream, mpsc::Sender<String>) {
            let split_addr: Vec<&str> = addr.split(":").collect();

            let ip = split_addr[0];
            let port = &(split_addr[1].parse::<u32>().unwrap() - 1).to_string();

            let listener = loop {
                let port = &(port.parse::<u32>().unwrap() + 1).to_string();
                let mut to_try: String = String::from(ip);
                to_try.push_str(":");
                to_try.push_str(port);

                match TcpListener::bind(to_try.clone()) {
                    Ok(l) => {
                        println!("Listening on: {}", to_try);
                        break l;
                    }
                    _ => {
                        println!("{} taken. Trying next port.", to_try);
                    }
                }
            };
            let (tx, rx) = mpsc::channel();
            (TcpStream{
                stream_buff: String::new(),
                listener: listener,
                rx: rx
            }, mpsc::Sender::clone(&tx))
        }

        pub fn handle(&mut self, sender: mpsc::Sender<String>) {
            for stream in self.listener.incoming() {
                match stream {
                    Ok(mut s) => {
                        let req = s.read_to_string(&mut self.stream_buff);
                        match req {
                            Ok(_) => {
                                sender.send(self.stream_buff.clone()).unwrap();
                                self.stream_buff.clear();
                            }
                            _ => println!("Err")
                        }
                        let to_send = self.rx.recv().unwrap().into_bytes();
                        s.write(&to_send).unwrap();
                    }
                    _ => println!("ERR")
                }
            }
        }
    }
}
