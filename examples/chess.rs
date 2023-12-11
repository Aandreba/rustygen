use autogen::{
    assistants::{chatgpt::ChatGPT, chess::ChessEngine},
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

    let mut i = 0;
    let mut conversation = MainConversation::<chess::Game>::new()
        .loop_while(|game| {
            println!("Round {i}");
            i += 1;
            game.result().is_none()
        })
        .agent(ChessEngine::new("./stockfish-ubuntu-x86-64", Duration::from_secs(1)).await?)
        .agent(ChatGPT::new("gpt-3.5-turbo", client))
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
