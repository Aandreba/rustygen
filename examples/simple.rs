use autogen::{assistants::chatgpt::ChatGPT, record::ChatRecord, Conversation};
use libopenai::Client;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let _ = color_eyre::install();
    dotenv::dotenv()?;
    let client = Client::new(None, None)?;

    let mut conversation = Conversation::<ChatRecord>::new();
    conversation
        .agent(String::from("Tell me about yourself"))
        .agent(ChatGPT::new("gpt-3.5-turbo", client));

    println!("{:#?}", conversation.play().await?);
    return Ok(());
}
