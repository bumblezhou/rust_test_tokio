use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {
    let socket = TcpStream::connect("127.0.0.1:6142").await?;
    let (mut reader, mut writer) = io::split(socket);

    let msg = "Hello world";

    tokio::spawn(async move {
        println!("Write data as bytes: {:?}", &msg.as_bytes());
        writer.write_all(&msg.as_bytes()).await?;
        println!("Write data: {:?}", &msg);

        Ok::<_, io::Error>(())
    });

    let mut buf = vec![0; 128];

    // let (mut socket, _) = listener.accept().await?;
    let n = reader.read(&mut buf).await?;
    println!("GOT as bytes {:?}", &buf[..n]);
    println!("GOT {:?}", std::str::from_utf8(&buf[..n]));

    Ok(())
}