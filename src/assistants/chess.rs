use crate::{agent::Agent, record::Record};
use chess::ChessMove;
use chessgineer::{game::Game, Context};
use failure::{Compat, Fail};
use std::{ffi::OsStr, str::FromStr, time::Duration};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Chess(Compat<chess::Error>),
    #[error("Illegal move: {0}")]
    IllegalMove(ChessMove),
}

pub struct ChessEngine {
    cx: Context,
    timeout: Duration,
}

impl ChessEngine {
    pub async fn new(path: impl AsRef<OsStr>, timeout: Duration) -> std::io::Result<Self> {
        return Ok(Self::with_context(Context::new(path).await?, timeout));
    }

    pub fn with_context(cx: Context, timeout: Duration) -> Self {
        return Self { cx, timeout };
    }
}

impl Record for chess::Game {
    type Error = Error;

    fn push(
        &mut self,
        _: libopenai::chat::Role,
        content: impl Into<crate::Str>,
    ) -> Result<(), Self::Error> {
        let content = content.into();
        let chess_move = ChessMove::from_str(&content).map_err(|e| Error::Chess(e.compat()))?;

        if !self.make_move(chess_move) {
            return Err(Error::IllegalMove(chess_move));
        }

        return Ok(());
    }
}

impl Agent<chess::Game> for ChessEngine {
    type Error = std::io::Error;

    async fn handle(&mut self, record: &mut chess::Game) -> Result<(), Self::Error> {
        let game = core::mem::replace(record, chess::Game::new());
        let mut engine_game = Game::with_game(game, &mut self.cx).await?;

        engine_game
            .calculate()
            .timeout(self.timeout)
            .start()
            .await?
            .make_best_move()
            .await?;

        *record = engine_game.into_state();
        return Ok(());
    }
}
