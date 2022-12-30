use client::Client;

use serde_repr::*;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{self, Sender};

use crate::message::Message;

mod client;
mod message;
mod request;
mod response;

const IP: &str = "192.168.1.100";
const PORT: u16 = 5000;

type GlobalState = Arc<Mutex<State>>;

#[derive(Default)]
pub struct State {
    pub tank_connected: bool,
    pub client_connected: bool,
}

impl State {
    fn new() -> GlobalState {
        Arc::new(Mutex::new(Self::default()))
    }
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone)]
#[repr(u8)]
pub enum Command {
    FORWARD,
    BACKWARD,
    LEFT,
    RIGHT,
    STOP,
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(format!("{}:{}", IP, PORT)).await.unwrap();
    println!("Server listening on port {}", PORT);

    let global_state = State::new();

    let (tx, _) = broadcast::channel::<Message>(5);

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        println!("New connection received");
        tokio::spawn(handle_client(socket, tx.clone(), global_state.clone()));
    }
}

async fn handle_client(mut socket: TcpStream, tx: Sender<Message>, state: Arc<Mutex<State>>) {
    let mut client = Client::new(&mut socket, tx, state);
    if let Err(e) = client.introduce().await {
        client.send_error(&e).await;
        return;
    };

    loop {
        tokio::select! {
            res = client.reader.read(&mut client.buffer) => {
                if !client.on_socket_read(res).await {
                    client.on_disconnect();
                    return;
                };

            }
            res = client.rx.recv() => {
                if !client.on_message_received(res).await {
                    client.on_disconnect();
                    return;
                };
            }
        }
    }
}
