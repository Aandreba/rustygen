use crate::Str;
use libopenai::chat::{Message, Role};

/// A record of the back-and-forth between the various agents
#[derive(Debug, Clone)]
pub struct Record {
    messages: Vec<Message<'static>>,
}

impl Record {
    pub fn new() -> Self {
        return Self {
            messages: Vec::new(),
        };
    }

    #[inline]
    pub fn messages(&self) -> &[Message<'static>] {
        return &self.messages;
    }

    #[inline]
    pub fn push(&mut self, role: Role, content: impl Into<Str>) {
        return self.messages.push(Message::new(role, content));
    }

    #[inline]
    pub fn push_message(&mut self, message: Message<'static>) {
        self.messages.push(message)
    }
}

impl Default for Record {
    fn default() -> Self {
        Self::new()
    }
}
