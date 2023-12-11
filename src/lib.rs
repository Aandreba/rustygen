#![feature(inline_const)]

use agent::{Agent, DynAgent};
use control_flow::r#while::{While, WhileBuilder};
use record::Record;
use std::borrow::Cow;

pub mod agent;
pub mod assistants;
pub mod control_flow;
pub mod record;

pub(crate) type Str = Cow<'static, str>;

/// Defines the structure of the desired conversation between agents
pub trait Conversation<'a, R: 'static + Record> {
    fn agent<A: 'a + Agent<R>>(self, agent: A) -> Self;

    fn while_loop<F: 'a + FnMut(&mut R) -> bool>(self, predicate: F) -> WhileBuilder<'a, R, F, Self>
    where
        Self: Sized,
    {
        return WhileBuilder {
            parent: self,
            child: While {
                predicate,
                conversation: MainConversation::new(),
            },
        };
    }
}

#[derive(Default)]
pub struct MainConversation<'a, R: 'static> {
    agents: Vec<DynAgent<'a, R>>,
}

impl<'a, R> MainConversation<'a, R> {
    /// Creates a new conversation
    pub fn new() -> Self {
        return Self { agents: Vec::new() };
    }
}

impl<'a, R: Record> MainConversation<'a, R> {
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

impl<'a, R: Record> Conversation<'a, R> for MainConversation<'a, R> {
    fn agent<A: 'a + Agent<R>>(mut self, agent: A) -> Self {
        self.agents.push(DynAgent::from_agent(agent));
        self
    }
}

impl<'a, R: Record> Agent<R> for MainConversation<'a, R> {
    type Error = color_eyre::Report;

    async fn handle(&mut self, record: &mut R) -> Result<(), Self::Error> {
        self.play_with(record).await
    }
}
