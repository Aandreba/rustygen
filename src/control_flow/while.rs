use crate::{agent::Agent, record::Record, Conversation};

pub struct While<'a, R: 'static, F> {
    pub(crate) predicate: F,
    pub(crate) conversation: Conversation<'a, R>,
}

impl<'a, R: Record, F: FnMut(&mut R) -> bool> Agent<R> for While<'a, R, F> {
    type Error = color_eyre::Report;

    async fn handle(&mut self, record: &mut R) -> Result<(), Self::Error> {
        while (self.predicate)(record) {
            self.conversation.play_with(record).await?;
        }
        return Ok(());
    }
}

pub struct WhileBuilder<'a, 'b, R: 'static, F> {
    pub(crate) parent: &'b mut Conversation<'a, R>,
    pub(crate) child: While<'a, R, F>,
}

impl<'a, 'b, R: Record, F: 'a + FnMut(&mut R) -> bool> WhileBuilder<'a, 'b, R, F> {
    pub fn agent<A: 'a + Agent<R>>(mut self, agent: A) -> Self {
        self.child.conversation.agent(agent);
        self
    }

    pub fn end_while(self) -> &'b mut Conversation<'a, R> {
        return self.parent.agent(self.child);
    }
}
