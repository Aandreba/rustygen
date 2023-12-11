use crate::{agent::Agent, record::Record, Conversation, MainConversation};

pub struct While<'a, R: 'static, F> {
    pub(crate) predicate: F,
    pub(crate) conversation: MainConversation<'a, R>,
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

pub struct WhileBuilder<'a, R: 'static, F, P> {
    pub(crate) parent: P,
    pub(crate) child: While<'a, R, F>,
}

impl<'a, R: Record, F: 'a + FnMut(&mut R) -> bool, P> Conversation<'a, R>
    for WhileBuilder<'a, R, F, P>
{
    fn agent<A: 'a + Agent<R>>(mut self, agent: A) -> Self {
        self.child.conversation = self.child.conversation.agent(agent);
        self
    }
}

impl<'a, R: Record, F: 'a + FnMut(&mut R) -> bool, P: Conversation<'a, R>>
    WhileBuilder<'a, R, F, P>
{
    pub fn end_while(self) -> P {
        return self.parent.agent(self.child);
    }
}
