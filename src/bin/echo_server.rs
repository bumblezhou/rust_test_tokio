use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await?;
    loop {
        let (mut socket, _) = listener.accept().await?;

        println!("echo_server accept.");

        tokio::spawn(async move {
            // Copy data here
            let (mut reader, mut writer) = socket.split();

            // if io::copy(&mut reader, &mut writer).await.is_err() {
            //     eprintln!("failed to copy");
            // }

            let mut buf = vec![0; 1024];
            loop {
                match reader.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(n) => {
                        println!("Received as bytes {:?}", &buf[..n]);
                        println!("Received {:?}", std::str::from_utf8(&buf[..n]));
                        if writer.write_all(&buf[..n]).await.is_err() {
                            return;
                        }
                    }
                    Err(_) => {
                        return;
                    }
                }
            }
        });
    }
}