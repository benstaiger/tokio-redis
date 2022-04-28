use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use mini_redis::Frame;
use mini_redis::Command::{self, Get, Set};
use tokio::net::{TcpListener, TcpStream};

use tokio_redis::connection::Connection;

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

async fn process(socket: TcpStream, db: Db) {
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let mut db = db.lock().unwrap();
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
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

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:6379";
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("Listening on {}.", addr);
    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db = db.clone();  // Clone the Arc<Mut<Hash>>
        
        println!("Accepted.");
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}