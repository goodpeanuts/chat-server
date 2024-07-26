use futures::{SinkExt, StreamExt};
use rust_chat_server::random_name;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Sender},
};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

const HELP_MSG: &str = include_str!("help.txt");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = TcpListener::bind("127.0.0.1:31888").await?;
    let (tx, _) = broadcast::channel::<String>(32);
    loop {
        let (tcp, _) = server.accept().await?;
        tokio::spawn(handle_user(tcp, tx.clone()));
    }
}

async fn handle_user(mut tcp: TcpStream, tx: Sender<String>) -> anyhow::Result<()> {
    let (reader, writer) = tcp.split();
    let mut stream = FramedRead::new(reader, LinesCodec::new());
    let mut sink = FramedWrite::new(writer, LinesCodec::new());
    let mut rx: broadcast::Receiver<String> = tx.subscribe();
    let user_name = random_name();
    sink.send(HELP_MSG).await?;
    sink.send(format!("You are {user_name}")).await?;
    loop {
        tokio::select! {
            user_msg = stream.next() => {
                let mut user_msg = match user_msg {
                    Some(msg) => msg?,
                    None => break,
                };
                if user_msg.starts_with("/help") {
                    sink.send(HELP_MSG).await?;
                    continue;
                } else if user_msg.starts_with("/quit") {
                    break;
                } else {
                    user_msg.push_str(" ❤️");
                    let _ = tx.send(format!("{user_name}: {user_msg}"));
                }
            },
            peer_msg = rx.recv() => {
                sink.send(peer_msg?).await?;
            }
        }
    }
    Ok(())
}
