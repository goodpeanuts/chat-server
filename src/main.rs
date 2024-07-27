use futures::{SinkExt, StreamExt};
use rust_chat_server::b;
use rust_chat_server::random_name;
use std::collections::HashSet;
use std::fmt::format;
use std::sync::{Arc, Mutex};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Sender},
};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

const HELP_MSG: &str = include_str!("help.txt");

#[derive(Clone)]
struct Names(Arc<Mutex<HashSet<String>>>);

impl Names {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(HashSet::new())))
    }
    fn insert(&self, name: String) -> bool {
        self.0.lock().unwrap().insert(name)
    }

    fn remove(&self, name: &str) -> bool {
        self.0.lock().unwrap().remove(name)
    }

    fn get_unique(&self) -> String {
        let mut name = random_name();
        let mut guard = self.0.lock().unwrap();
        while !guard.insert(name.clone()) {
            name = random_name();
        }
        name
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = TcpListener::bind("127.0.0.1:31888").await?;
    let (tx, _) = broadcast::channel::<String>(32);
    let names = Names::new();
    loop {
        let (tcp, _) = server.accept().await?;
        tokio::spawn(handle_user(tcp, tx.clone(), names.clone()));
    }
}

async fn handle_user(mut tcp: TcpStream, tx: Sender<String>, names: Names) -> anyhow::Result<()> {
    let (reader, writer) = tcp.split();
    let mut stream = FramedRead::new(reader, LinesCodec::new());
    let mut sink = FramedWrite::new(writer, LinesCodec::new());
    let mut rx: broadcast::Receiver<String> = tx.subscribe();
    let mut user_name = names.get_unique();
    sink.send(format!("{HELP_MSG}\nYou are {user_name}"))
        .await?;
    let result = loop {
        tokio::select! {
            user_msg = stream.next() => {
                let mut user_msg = match user_msg {
                    Some(msg) => b!(msg),
                    None => break Ok(()),
                };
                if user_msg.starts_with("/help") {
                    b!(sink.send(HELP_MSG).await);
                    continue;
                } else if user_msg.starts_with("/quit") {
                    break Ok(());
                } else if user_msg.starts_with("/name") {
                    let new_name = match user_msg.
                        split_ascii_whitespace().
                        nth(1) {
                            Some(new_name) => {
                                new_name.to_owned()
                            },
                            None => {
                                b!(sink.send("Name must be 1 - 20 alphanumeric chars").await);
                                continue;
                            }
                        };
                    let changed_name = names.insert(new_name.clone());
                    if changed_name {
                        b!(tx.send(format!("{user_name} is now {new_name}")));
                        names.remove(&user_name);
                        user_name = new_name;
                    } else {
                        b!(sink.send(format!("{new_name} is already taken")).await);
                    }
                } else {
                    user_msg.push_str(" ❤️");
                    let _ = tx.send(format!("{user_name}: {user_msg}"));
                }
            },
            peer_msg = rx.recv() => {
                let peer_msg = b!(peer_msg);
                b!(sink.send(peer_msg).await);
            }
        }
    };
    let _ = tx.send(format!("{user_name} offline"));
    names.remove(&user_name);
    result
}
