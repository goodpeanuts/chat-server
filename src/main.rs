use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener}};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = TcpListener::bind("127.0.0.1:31888").await?;
    loop {
        let (mut tcp, cli) = server.accept().await?;
        println!("@ accept {cli}");
        let mut buffer = [0u8; 16];
        loop {
            let n = tcp.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            let _ = tcp.write_all(&buffer[..n]).await?;

            let mut line = String::from_utf8(buffer[..n].to_vec())?;
            line.pop();
            line.pop();
            line.push_str("❤️\n");
            let _ = tcp.write_all(line.as_bytes()).await?;
        }
    }
    // Ok(())
}
