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
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChessError {
    #[error("{0}")]
    ChatGpt(#[from] Error),
    #[error("{0}")]
    Chess(#[from] failure::Compat<chess::Error>),
    #[error("No legal move found")]
    NoLegalMoveFound,
}

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

    pub fn into_chess(self, max_tries: usize) -> ChessGPT {
        return ChessGPT {
            client: self.client,
            model: self.model,
            max_tries,
        };
    }
}

impl Agent<ChatRecord> for ChatGPT {
    type Error = Error;

    async fn handle(&mut self, record: &mut ChatRecord) -> Result<(), Self::Error> {
        return self.handle_ref(record).await;
    }
}

impl AgentRef<ChatRecord> for ChatGPT {
    async fn handle_ref(&self, record: &mut ChatRecord) -> Result<(), Self::Error> {
        let mut chat =
            ChatCompletion::new(self.model.deref(), record.messages().to_vec(), &self.client)
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

/// Chess-specialized verision of [`ChatGPT`]
pub struct ChessGPT {
    pub client: Client,
    pub model: Str,
    pub max_tries: usize,
}

impl ChessGPT {
    pub fn new(model: impl Into<Str>, client: Client, max_tries: usize) -> Self {
        return Self {
            client,
            model: model.into(),
            max_tries,
        };
    }

    pub fn into_chat(self) -> ChatGPT {
        return ChatGPT {
            client: self.client,
            model: self.model,
        };
    }
}

impl Agent<chess::Game> for ChessGPT {
    type Error = ChessError;

    async fn handle(&mut self, record: &mut chess::Game) -> Result<(), Self::Error> {
        return self.handle_ref(record).await;
    }
}

impl AgentRef<chess::Game> for ChessGPT {
    async fn handle_ref(&self, record: &mut chess::Game) -> Result<(), Self::Error> {
        let mut is_assistant = record.side_to_move() == Color::White;
        let mut messages = Vec::with_capacity(2 * record.actions().len());

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

        let mut illegal_moves = Vec::with_capacity(self.max_tries);
        for _ in 0..self.max_tries {
            let mut messages = messages.clone();
            messages.push(Message::system(
                format!("You're a chess engine. Respond only with the next move to play, based on the previous moves{}, using the UCI format. The current state of the board is {} (using FEN notation).",
                illegal_moves.is_empty().then(|| format!(" and knowing ({}) are illegal moves.",
                illegal_moves.join(", "))).unwrap_or_default(),
                record.current_position().to_string()),
            ));

            let mut chat =
                ChatCompletion::new(self.model.deref(), messages.clone(), &self.client).await?;

            if chat.choices.is_empty() {
                return Err(ChessError::ChatGpt(Error::msg("No response choices found")));
            }

            let mut msg = chat.choices.swap_remove(0).message;
            println!("{msg:?}");

            // Transform response into valid UCI move format
            if msg.content.ends_with(|c: char| !c.is_alphanumeric()) {
                let _ = msg.content.to_mut().pop();
            }
            msg.content = Str::Owned(
                msg.content
                    .split_whitespace()
                    .last()
                    .unwrap_or(&msg.content)
                    .to_string(),
            );

            match record.push_message(msg) {
                Ok(_) => return Ok(()),
                Err(super::chess::Error::IllegalMove(chess_move)) => {
                    illegal_moves.push(chess_move.to_string());
                    continue;
                }
                Err(super::chess::Error::Chess(e)) => {
                    return Err(ChessError::Chess(e));
                }
            }
        }

        return Err(ChessError::NoLegalMoveFound);
    }
}
