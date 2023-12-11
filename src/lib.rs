#![feature(inline_const)]

use agent::{Agent, DynAgent};
use record::Record;
use std::borrow::Cow;

pub mod agent;
pub mod assistants;
pub mod record;

pub(crate) type Str = Cow<'static, str>;

#[derive(Default)]
pub struct Conversation<'a> {
    agents: Vec<DynAgent<'a>>,
}

impl<'a> Conversation<'a> {
    pub fn new() -> Self {
        return Self { agents: Vec::new() };
    }

    pub fn agent<A: 'a + Agent>(&mut self, agent: A) -> &mut Self {
        self.agents.push(DynAgent::from(agent));
        self
    }

    pub async fn play(&mut self) -> color_eyre::Result<Record> {
        let mut record = Record::new();
        for agent in self.agents.iter_mut() {
            agent.handle(&mut record).await?
        }

        return Ok(record);
    }
}
