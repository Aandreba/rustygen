# rustygen - A Rusty version of autogen

## Examples

**Basic**

```rust
use rustygen::{assistants::gpt::ChatGPT, record::ChatRecord, Conversation, MainConversation};
use libopenai::Client;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let _ = color_eyre::install();
    dotenv::dotenv()?;
    let client = Client::new(None, None)?;

    let mut conversation = MainConversation::<ChatRecord>::new()
        .agent(String::from("Tell me about yourself"))
        .agent(ChatGPT::new("gpt-3.5-turbo", client));

    println!("{:#?}", conversation.play().await?);
    return Ok(());
}

```

**Chess**

```rust
use rustygen::{
    agent::Agent,
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

    let mut i = 0;
    let mut conversation = MainConversation::<chess::Game>::new()
        .while_loop(|game| {
            println!("Round {i}");
            i += 1;
            game.result().is_none()
        })
        .agent(ChessEngine::new("./stockfish-ubuntu-x86-64", Duration::from_secs(1)).await?)
        .agent(
            ChessGPT::new("gpt-3.5-turbo", client, 5).catch(|e| async move {
                match e {
                    // Fall back to Stockfish whenever ChatGPT fails generating a legal move
                    ChessError::NoLegalMoveFound => {
                        println!("Resorting to Stockfish for GPT's move");
                        Ok(
                            ChessEngine::new("./stockfish-ubuntu-x86-64", Duration::from_secs(1))
                                .await
                                .unwrap(),
                        )
                    }
                    e => Err(e),
                }
            }),
        )
        .end_while();

    let mut game = chess::Game::new();
    if let Err(e) = conversation.play_with(&mut game).await {
        eprintln!("{e}");
    }

    for action in game.actions() {
        match action {
            Action::MakeMove(x) => println!("{x}"),
            other => println!("{other:?}"),
        }
    }

    println!("{}", game.current_position().to_string());
    return Ok(());
}
```
