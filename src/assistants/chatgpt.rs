use crate::{
    agent::{Agent, AgentRef},
    record::{ChatRecord, Record},
    Str,
};
use chess::Color;
use libopenai::{
    chat::{ChatCompletion, Message, Role},
    error::Error,
    Client,
};
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

/* Agents */
impl Agent<ChatRecord> for ChatGPT {
    type Error = Error;

    async fn handle(&mut self, record: &mut ChatRecord) -> Result<(), Self::Error> {
        return self.handle_ref(record).await;
    }
}

impl Agent<chess::Game> for ChatGPT {
    type Error = Error;

    async fn handle(&mut self, record: &mut chess::Game) -> Result<(), Self::Error> {
        return self.handle_ref(record).await;
    }
}

/* Ref Agents */
impl AgentRef<ChatRecord> for ChatGPT {
    async fn handle_ref(&self, record: &mut ChatRecord) -> Result<(), Self::Error> {
        let mut chat = ChatCompletion::new(
            self.model.deref(),
            record.messages().iter().cloned(),
            &self.client,
        )
        .await?;

        if chat.choices.is_empty() {
            return Err(Error::msg("No response choices found"));
        }

        record
            .push_message(chat.choices.swap_remove(0).message)
            .unwrap();
        return Ok(());
    }
}

impl AgentRef<chess::Game> for ChatGPT {
    async fn handle_ref(&self, record: &mut chess::Game) -> Result<(), Self::Error> {
        let mut messages = vec![Message::system(
            "You're a chess engine. Respond only with the next move to play, based on the previous moves, using the UCI format.",
        )];

        let mut is_assistant = record.side_to_move() == Color::White;
        messages.reserve(record.actions().len());
        for action in record.actions() {
            if let chess::Action::MakeMove(chess_move) = action {
                messages.push(Message::new(
                    is_assistant
                        .then_some(Role::Assistant)
                        .unwrap_or(Role::User),
                    chess_move.to_string(),
                ));
                is_assistant = !is_assistant
            }
        }

        let mut chat = ChatCompletion::new(self.model.deref(), messages, &self.client).await?;
        if chat.choices.is_empty() {
            return Err(Error::msg("No response choices found"));
        }

        record
            .push_message(chat.choices.swap_remove(0).message)
            .map_err(|e| Error::msg(failure::Error::from(e)))?;
        return Ok(());
    }
}
