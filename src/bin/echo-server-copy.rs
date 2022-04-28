use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            // A stack buffer is explicitly avoided. Recall from earlier, we
            // noted that all task data that lives across calls to .await must
            // be stored by the task. In this case, buf is used across .await
            // calls. All task data is stored in a single allocation.
            // [source](https://tokio.rs/tokio/tutorial/io)

            // If a stack array is used as the buffer type, it will be stored
            // inline in the task structure. This will make the task structure
            // very big. Additionally, buffer sizes are often page sized. This
            // will, in turn, make Task an awkward size: $page-size +
            // a-few-bytes.

            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(n) => {
                        if socket.write_all(&buf[..n]).await.is_err() {
                            // unexpected socket error.
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
