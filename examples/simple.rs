use autogen::{assistants::gpt::ChatGPT, record::ChatRecord, Conversation, MainConversation};
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
