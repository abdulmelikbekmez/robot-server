use serde::Serialize;

use crate::Command;

#[derive(Serialize)]
pub enum Response {
    Action(Command),
    Er(String),
}
