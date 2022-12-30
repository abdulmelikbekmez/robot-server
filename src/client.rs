use serde_repr::*;
use std::io::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpStream,
    },
    sync::broadcast::{error::RecvError, Receiver, Sender},
};

use crate::{message::Message, request::Request, response::Response, GlobalState};

#[derive(Serialize_repr, Deserialize_repr, Debug)]
#[repr(u8)]
pub enum ClientType {
    USER,
    TANK,
    UNDEFINED,
}

pub struct Client<'a> {
    pub t: ClientType,
    pub reader: BufReader<ReadHalf<'a>>,
    pub writer: WriteHalf<'a>,
    pub tx: Sender<Message>,
    pub rx: Receiver<Message>,
    pub state: GlobalState,
    pub buffer: [u8; 1024],
}

impl<'a> Client<'a> {
    pub fn new(socket: &'a mut TcpStream, tx: Sender<Message>, state: GlobalState) -> Self {
        let (read, writer) = socket.split();
        let reader = BufReader::new(read);
        let rx = tx.subscribe();
        Self {
            t: ClientType::UNDEFINED,
            reader,
            writer,
            tx,
            rx,
            state,
            buffer: [0; 1024],
        }
    }

    pub async fn introduce(&mut self) -> Result<(), String> {
        println!("introduce");
        match self.reader.read(&mut self.buffer).await {
            Ok(0) => Err("client disconnected".to_string()),
            Ok(size) => {
                let Request::Introduce(c) = Request::from_slice(&self.buffer, size)? else {
                    return Err("Wrong Request Type".to_string());
            };
                self.set_client_type(c).await?;
                Ok(())
            }
            Err(_) => Err("reader error".to_string()),
        }
    }

    pub async fn on_socket_read(&mut self, res: Result<usize, Error>) -> bool {
        println!("on socket read");
        match res {
            Ok(0) => {
                println!("connection closed due to 0 byte readed");
                false
            }
            Ok(size) => match Request::from_slice(&self.buffer, size) {
                Ok(request) => match request {
                    Request::Action(command) => {
                        let message = Message::Command(command);
                        self.send_message(message).await;
                        true
                    }
                    _ => {
                        println!("action must be received!!");
                        false
                    }
                },
                Err(_) => false,
            },
            Err(_) => false,
        }
    }

    pub async fn on_message_received(&mut self, res: Result<Message, RecvError>) -> bool {
        let Ok(message) = res else {
            return false;
        };

        match message {
            Message::Command(command) => {
                let response = Response::Action(command);
                self.send_response(&response).await;
                true
            }
        }
    }

    pub fn on_disconnect(&mut self) {
        println!("on disconnect");
        match self.t {
            ClientType::USER => self.set_client_status(false),
            ClientType::TANK => self.set_tank_status(false),
            ClientType::UNDEFINED => println!("undefined disconnected"),
        };
        println!("status after disconnect => {}", self.get_client_status());
    }

    async fn set_client_type(&mut self, t: ClientType) -> Result<(), String> {
        match t {
            ClientType::USER => {
                if self.get_client_status() {
                    return Err("Client Already Connected!!".to_string());
                }
                self.t = t;
                self.set_client_status(true);
                println!("Client Connected!!");
                Ok(())
            }
            ClientType::TANK => {
                if self.get_tank_status() {
                    return Err("Tank Already Connected!!".to_string());
                }
                self.t = t;
                self.set_tank_status(true);
                println!("Tank Connected!!");
                Ok(())
            }
            ClientType::UNDEFINED => Err("Wrong Client Type received".to_string()),
        }
    }
    pub async fn send_error(&mut self, error: &str) {
        println!("error sended {}", error);
        self.send_response(&Response::Er(error.to_string())).await;
    }

    async fn send_response(&mut self, response: &Response) {
        let buf = serde_json::to_vec(&response).unwrap();
        self.writer.write_all(buf.as_slice()).await.unwrap();
    }

    async fn send_message(&mut self, message: Message) {
        println!("message sended to other command");
        self.tx.send(message).unwrap();
    }

    fn set_tank_status(&mut self, status: bool) {
        self.state.lock().unwrap().tank_connected = status;
    }

    fn get_tank_status(&self) -> bool {
        self.state.lock().unwrap().tank_connected
    }

    fn set_client_status(&mut self, status: bool) {
        self.state.lock().unwrap().client_connected = status;
    }

    fn get_client_status(&self) -> bool {
        self.state.lock().unwrap().client_connected
    }
}
