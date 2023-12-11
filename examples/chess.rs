use autogen::{
    assistants::{chatgpt::ChatGPT, chess::ChessEngine},
    Conversation,
};
use libopenai::Client;
use std::time::Duration;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let _ = color_eyre::install();
    dotenv::dotenv()?;
    let client = Client::new(None, None)?;

    let mut conversation = Conversation::<chess::Game>::new();
    conversation
        .agent(ChessEngine::new("./stockfish-ubuntu-x86-64", Duration::from_secs(1)).await?)
        .agent(ChatGPT::new("gpt-3.5-turbo", client));

    let mut game = chess::Game::new();
    conversation.play_with(&mut game).await?;
    println!("{game:#?}");

    return Ok(());
}
