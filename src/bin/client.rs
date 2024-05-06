use mini_redis::client;
use bytes::Bytes;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        value: Bytes,
        resp: Responder<()>
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get { key: "foo".to_string(), resp: resp_tx };
        
        tx.send(cmd).await.unwrap();

        let res = resp_rx.await;
        println!("t1 get resp GOT = {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set { key: "foo".to_string(), value: "bar".into(), resp: resp_tx };

        tx2.send(cmd).await.unwrap();

        let res = resp_rx.await;
        println!("t2 set resp GOT = {:?}", res);
    });

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            use Command::*;
            match cmd {
                Get { key, resp } => {
                    let value = client.get(&key).await;
                    println!("Got value: {:?}", &value);
                    let _ = resp.send(value);
                }
                Set { key, value, resp } => {
                    let res = client.set(&key, value.clone()).await;
                    println!("Set value: {:?} -> {:?}", key, value);
                    let _ = resp.send(res);
                }
                
            }
        }
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
    
}