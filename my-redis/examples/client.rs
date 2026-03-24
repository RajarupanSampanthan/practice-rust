use bytes::Bytes;
use core::hash::{Hash, Hasher};
use std::collections::HashMap;
use std::hash::DefaultHasher;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};

use mini_redis::{Connection, Frame, client};
use tokio::net::{TcpListener, TcpStream};

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
    },
}

/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
    // Establish a connection to the server

    let (tx, mut rx) = mpsc::channel::<Command>(32);

    let tx2 = tx.clone();

    let task_1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        tx.send(Command::Get {
            key: "foo".into(),
            resp: resp_tx,
        })
        .await
        .unwrap();

        let res = resp_rx.await.unwrap();
        println!("GOT = {:?}", res);
    });

    let task_2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        tx2.send(Command::Set {
            key: "foo".into(),
            val: "bar".into(),
            resp: resp_tx,
        })
        .await
        .unwrap();

        // Await the response
        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });

    let manager = tokio::spawn(async move {
        // Establish a connection to the server
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        // Start receiving messages
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key, resp } => {
                    let res = client.get(&key).await;
                    let _ = resp.send(res);
                }
                Command::Set { key, val, resp } => {
                    let res = client.set(&key, val).await;
                    let _ = resp.send(res);
                }
            }
        }
    });

    task_2.await.unwrap();
    task_1.await.unwrap();

    manager.await.unwrap();
}
