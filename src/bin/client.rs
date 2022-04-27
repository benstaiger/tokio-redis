use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    }
}

#[tokio::main]
async fn main() {
    let (prod, mut cons) = mpsc::channel(32);
    let prod2 = prod.clone(); // Not to self, why doesn't the producer need to be mut?

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = cons.recv().await {
            match cmd {
                // why doesn't this find the command in the scope?
                Command::Get { key, resp } => {
                    let res = client.get(&key).await;
                    let _ = resp.send(res);
                }
                Command::Set {key, val, resp } => {
                    let res = client.set(&key, val).await;
                    let _ = resp.send(res);
                }
            }
        }
    });

    let t1 = tokio::spawn(async move {
        let (resp, recv) = oneshot::channel();
        let cmd = Command::Get {
            key: "hello".to_string(),
            resp,
        };

        prod.send(cmd).await.unwrap();
        let res = recv.await;
        println!("GOT = {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (resp, recv) = oneshot::channel();
        let cmd = Command::Set { 
            key: "foo".to_string(),
            val: "bar".into(),
            resp,
        };
        prod2.send(cmd).await.unwrap();

        let res = recv.await;
        println!("GOT {:?}", res);
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}