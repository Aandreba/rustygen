use crate::Str;
use libopenai::chat::{Message, Role};
use std::convert::Infallible;

/// A record of the back-and-forth between the various agents
pub trait Record: 'static {
    type Error;

    fn push(&mut self, role: Role, content: impl Into<Str>) -> Result<(), Self::Error>;

    #[inline]
    fn push_message(&mut self, message: Message<'static>) -> Result<(), Self::Error> {
        self.push(message.role, message.content)
    }
}

/// A chat-based record
#[derive(Debug, Clone)]
pub struct ChatRecord {
    messages: Vec<Message<'static>>,
}

impl ChatRecord {
    pub fn new() -> Self {
        return Self {
            messages: Vec::new(),
        };
    }

    #[inline]
    pub fn messages(&self) -> &[Message<'static>] {
        return &self.messages;
    }
}

impl Record for ChatRecord {
    type Error = Infallible;

    #[inline]
    fn push(&mut self, role: Role, content: impl Into<Str>) -> Result<(), Self::Error> {
        self.messages.push(Message::new(role, content));
        return Ok(());
    }

    #[inline]
    fn push_message(&mut self, message: Message<'static>) -> Result<(), Self::Error> {
        self.messages.push(message);
        return Ok(());
    }
}

impl Default for ChatRecord {
    fn default() -> Self {
        Self::new()
    }
}
