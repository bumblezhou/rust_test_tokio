use mini_redis::client;
use bytes::Bytes;
use tokio::sync::mpsc;

#[derive(Debug)]
enum Command {
    Get {
        key: String
    },
    Set {
        key: String,
        value: Bytes
    }
}

#[tokio::main]
async fn main() {
    // let mut client = client::connect("127.0.0.1:6379").await.unwrap();

    // let t1 = tokio::spawn(async {
    //     let res = client.get("foo").await;
    // });

    // let t2 = tokio::spawn(async {
    //     client.set("foo", "bar".into()).await;
    // });

    // t1.await.unwrap();
    // t2.await.unwrap();

    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    let t1 = tokio::spawn(async move {
        let cmd = Command::Get { key: "foo".to_string() };
        tx.send(cmd).await.unwrap();
    });

    let t2 = tokio::spawn(async move {
        let cmd = Command::Set { key: "foo".to_string(), value: "bar".into() };
        tx2.send(cmd).await.unwrap();
    });

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            use Command::*;
            match cmd {
                Get { key } => {
                    let value = client.get(&key).await;
                    println!("Got value: {:?}", value);
                }
                Set { key, value } => {
                    let _ = client.set(&key, value.clone()).await;
                    println!("Set value: {:?} -> {:?}", key, value)
                }
                
            }
        }
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
    
}