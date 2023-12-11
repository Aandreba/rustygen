use autogen::{
    agent::{Agent, AgentRef},
    assistants::{
        chess::ChessEngine,
        gpt::{ChessError, ChessGPT},
    },
    Conversation, MainConversation,
};
use chess::Action;
use libopenai::Client;
use std::time::Duration;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let _ = color_eyre::install();
    dotenv::dotenv()?;
    let client = Client::new(None, None)?;

    // Backup stockfish engin for whenever ChatGPT fails generating a legal move
    let mut backup_stockfish =
        ChessEngine::new("./stockfish-ubuntu-x86-64", Duration::from_secs(1)).await?;

    let mut i = 0;
    let mut conversation = MainConversation::<chess::Game>::new()
        .while_loop(|game| {
            println!("Round {i}");
            i += 1;
            game.result().is_none()
        })
        .agent(ChessEngine::new("./stockfish-ubuntu-x86-64", Duration::from_secs(1)).await?)
        .agent(
            ChessGPT::new("gpt-3.5-turbo", client, 5).catch(|e| match e {
                ChessError::NoLegalMoveFound => {
                    println!("Resorting to Stockfish for GPT's move");
                    Ok(&mut backup_stockfish)
                }
                e => Err(e),
            }),
        )
        .end_while();

    let mut game = chess::Game::new();
    let _ = conversation.play_with(&mut game).await;

    for action in game.actions() {
        match action {
            Action::MakeMove(x) => println!("{x}"),
            other => println!("{other:?}"),
        }
    }

    return Ok(());
}
