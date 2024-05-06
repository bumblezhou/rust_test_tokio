use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};
use mini_redis::Command::{self, Get, Set};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// type Db = Arc<Mutex<HashMap<String, Bytes>>>;
type ShardedDb = Arc<Vec<Mutex<HashMap<String, Vec<u8>>>>>;

fn new_sharded_db(num_shards: usize) -> ShardedDb {
    let mut db = Vec::with_capacity(num_shards);
    for _ in 0..num_shards {
        db.push(Mutex::new(HashMap::new()));
    }
    Arc::new(db)
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    println!("Listening...");

    // let db = Arc::new(Mutex::new(HashMap::new()));
    let db = new_sharded_db(5);

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // 1st version, await and process
        // process(socket).await;

        let db = db.clone();

        println!("Accepted");

        // 2nd version, spawn new task to process
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

fn get_hash_of_str(key: &String) -> usize {
    let key = key;
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hash = hasher.finish();
    hash as usize
}

async fn process(socket: TcpStream, db: ShardedDb) {
    // let mut db = HashMap::new();

    let mut connection = Connection::new(socket);
    while let Some(frame) = connection.read_frame().await.unwrap() {
        // println!("GOT: {:?}", frame);

        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                // let mut db = db.lock().unwrap();
                let hash = get_hash_of_str(&cmd.key().to_string());
                let shard_index = hash % 5;
                let mut shard = db[shard_index].lock().unwrap();
                shard.insert(cmd.key().to_string(), cmd.value().clone().to_vec());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                // let db = db.lock().unwrap();
                let hash = get_hash_of_str(&cmd.key().to_string());
                let shard_index = hash % 5;
                let shard = db[shard_index].lock().unwrap();
                if let Some(value) = shard.get(cmd.key()) {
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };
        connection.write_frame(&response).await.unwrap();
    }
}