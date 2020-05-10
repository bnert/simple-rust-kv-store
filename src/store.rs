pub mod store {
    use std::sync::{mpsc, RwLock};
    use std::collections::HashMap;
    use serde::{Deserialize, Serialize};


    #[derive(Serialize, Deserialize)]
    enum EventType {
        GET,
        PUT,
        DEL,
        LST,
        ERR
    }

    #[derive(Serialize, Deserialize)]
    struct Event {
        action: EventType,
        key: String,
        data: Option<String>
    }

    pub struct KeyValueStore {
        backend: RwLock<HashMap<String, String>>,
        rx: mpsc::Receiver<String>,
    }

    impl KeyValueStore {
        pub fn new() -> (KeyValueStore, mpsc::Sender<String>) {
            let (tx, rx) = mpsc::channel();
            (KeyValueStore {
                backend: RwLock::new(HashMap::new()),
                rx: rx
            }, mpsc::Sender::clone(&tx))
        }

        pub fn handle(&mut self, sender: mpsc::Sender<String>) {
            for m in &self.rx {
                let e: Event = match serde_json::from_str(&m) {
                    Ok(v) => v,
                    _ => Event{ action: EventType::ERR, key: String::from(""), data: None }
                };
                match e.action {
                    EventType::ERR => {
                        sender.send(String::from("error")).unwrap();
                    }
                    EventType::PUT => {
                        // Not a read, so lock the table
                        let mut s = self.backend.write().unwrap();
                        match s.insert(e.key, e.data.unwrap_or("".to_string())) {
                            Some(_) => sender.send(String::from("updated")).unwrap(),
                            None => sender.send(String::from("created")).unwrap(),
                        }
                    }
                    EventType::GET => {
                        // Read, so no lock
                        let s = self.backend.read().unwrap();
                        match s.get(&e.key) {
                            Some(v) => sender.send(v.clone()).unwrap(),
                            None => sender.send(String::from("undefined")).unwrap(),
                        }
                    }
                    EventType::DEL => {
                        // Lock
                        let mut s = self.backend.write().unwrap();
                        match s.remove_entry(&e.key) {
                            Some(_) => sender.send(String::from("ok")).unwrap(),
                            None => sender.send(String::from("err")).unwrap(),
                        }
                    }
                   EventType::LST => {
                       // No lock
                       sender.send(String::from("not impl")).unwrap();
                   }
                }
            }
        }
    }
}

