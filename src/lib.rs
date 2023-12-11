#![feature(inline_const)]

use agent::{Agent, DynAgent};
use record::Record;
use std::borrow::Cow;

pub mod agent;
pub mod assistants;
pub mod record;

pub(crate) type Str = Cow<'static, str>;

#[derive(Default)]
pub struct Conversation<'a, R: 'static> {
    agents: Vec<DynAgent<'a, R>>,
}

impl<'a, R> Conversation<'a, R> {
    pub fn new() -> Self {
        return Self { agents: Vec::new() };
    }
}

impl<'a, R: Record> Conversation<'a, R> {
    pub fn agent<A: 'a + Agent<R>>(&mut self, agent: A) -> &mut Self {
        self.agents.push(DynAgent::from_agent(agent));
        self
    }

    pub async fn play(&mut self) -> color_eyre::Result<R>
    where
        R: Default,
    {
        let mut record = R::default();
        self.play_with(&mut record).await?;
        return Ok(record);
    }

    pub async fn play_with(&mut self, record: &mut R) -> color_eyre::Result<()> {
        for agent in self.agents.iter_mut() {
            agent.handle(record).await?
        }
        return Ok(());
    }
}
