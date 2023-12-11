use crate::{
    agent::{Agent, AgentRef},
    record::Record,
};
use std::future::Future;

/// Catches the error of it's parent agent, and handles it
pub struct Catch<A, F> {
    pub(crate) agent: A,
    pub(crate) f: F,
}

impl<
        R: Record,
        A: Agent<R>,
        F: FnMut(A::Error) -> Fut,
        Fut: Future<Output = Result<A2, A::Error>>,
        A2: Agent<R>,
    > Agent<R> for Catch<A, F>
{
    type Error = color_eyre::Report;

    async fn handle(&mut self, record: &mut R) -> Result<(), Self::Error> {
        if let Err(e) = self.agent.handle(record).await {
            return match (self.f)(e).await {
                Ok(mut agent) => agent.handle(record).await.map_err(Into::into),
                Err(e) => Err(e.into()),
            };
        }
        return Ok(());
    }
}

impl<
        R: Record,
        A: AgentRef<R>,
        F: Fn(A::Error) -> Fut,
        Fut: Future<Output = Result<A2, A::Error>>,
        A2: Agent<R>,
    > AgentRef<R> for Catch<A, F>
{
    async fn handle_ref(&self, record: &mut R) -> Result<(), Self::Error> {
        if let Err(e) = self.agent.handle_ref(record).await {
            return match (self.f)(e).await {
                Ok(mut agent) => agent.handle(record).await.map_err(Into::into),
                Err(e) => Err(e.into()),
            };
        }
        return Ok(());
    }
}
