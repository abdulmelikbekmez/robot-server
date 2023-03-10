use serde::Deserialize;

use crate::{client::ClientType, Command};

#[derive(Deserialize, Debug)]
pub enum Request {
    Introduce(ClientType),
    Action(Command),
}

impl Request {
    pub fn from_slice(buffer: &[u8], size: usize) -> Result<Self, String> {
        println!("size => {}", size);
        let tmp = &buffer[..size];

        let s = String::from_utf8(tmp.to_vec());
        println!("input => {:?}", s);

        let res = serde_json::from_slice::<Request>(tmp)
            .map_err(|e| format!("deserialize error: {}", e.to_string()))?;

        println!("request => {:?}", res);
        return Ok(res);
    }
}
