use crate::{
    agent::{Agent, AgentRef},
    Str,
};
use libopenai::{chat::ChatCompletion, error::Error, Client};
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct ChatGPT {
    pub client: Client,
    pub model: Str,
}

impl ChatGPT {
    pub fn new(model: impl Into<Str>, client: Client) -> Self {
        return Self {
            client,
            model: model.into(),
        };
    }
}

impl Agent for ChatGPT {
    type Error = Error;

    async fn handle(&mut self, record: &mut crate::record::Record) -> Result<(), Self::Error> {
        return self.handle_ref(record).await;
    }
}

impl AgentRef for ChatGPT {
    async fn handle_ref(&self, record: &mut crate::record::Record) -> Result<(), Self::Error> {
        let mut chat = ChatCompletion::new(
            self.model.deref(),
            record.messages().iter().cloned(),
            &self.client,
        )
        .await?;

        if chat.choices.is_empty() {
            return Err(Error::msg("No response choices found"));
        }

        record.push_message(chat.choices.swap_remove(0).message);
        return Ok(());
    }
}
