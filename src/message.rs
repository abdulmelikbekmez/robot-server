use crate::Command;

#[derive(Debug, Clone)]
pub enum Message {
    Command(Command),
}
