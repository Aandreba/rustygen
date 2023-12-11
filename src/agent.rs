use crate::record::Record;
use libopenai::chat::Role;
use std::{future::Future, marker::PhantomData, pin::Pin};

/// TODO
pub trait Agent<R: Record> {
    type Error: Into<color_eyre::Report>;

    async fn handle(&mut self, record: &mut R) -> Result<(), Self::Error>;
}

/// An agent that can be executed through shared reference
pub trait AgentRef<R: Record>: Agent<R> {
    async fn handle_ref(&self, record: &mut R) -> Result<(), Self::Error>;
}

/* BLANKET IMPLS */
impl<R: Record, A: Agent<R>> Agent<R> for &mut A {
    type Error = A::Error;

    async fn handle(&mut self, record: &mut R) -> Result<(), Self::Error> {
        A::handle(self, record).await
    }
}

impl<R: Record, A: AgentRef<R>> Agent<R> for &A {
    type Error = A::Error;

    async fn handle(&mut self, record: &mut R) -> Result<(), Self::Error> {
        A::handle_ref(self, record).await
    }
}

/* DEFAULT IMPLS */

/// `String`s can be directly used as a [`User`](Role::User) message
impl<R: Record> Agent<R> for String
where
    R::Error: 'static + std::error::Error + Send + Sync,
{
    type Error = R::Error;

    async fn handle(&mut self, record: &mut R) -> Result<(), Self::Error> {
        return record.push(Role::User, self.clone());
    }
}

impl<R: Record> AgentRef<R> for String
where
    R::Error: 'static + std::error::Error + Send + Sync,
{
    async fn handle_ref(&self, record: &mut R) -> Result<(), Self::Error> {
        return record.push(Role::User, self.clone());
    }
}

/* DYNAMIC AGENT */

pub(crate) struct DynAgent<'a, R: 'static> {
    data: *mut (),
    vtable: &'static DynAgentVTable<R>,
    _phtm: PhantomData<&'a mut &'a ()>,
}

impl<'a, R: Record> DynAgent<'a, R> {
    pub fn handle<'b>(
        &'b mut self,
        record: &'b mut R,
    ) -> Pin<Box<dyn 'b + Future<Output = color_eyre::Result<()>>>> {
        return (self.vtable.handle)(self.data, record);
    }
}

impl<'a, R: Record> DynAgent<'a, R> {
    pub fn from_agent<A: 'a + Agent<R>>(agent: A) -> Self {
        return Self::from_boxed_agent(Box::new(agent));
    }

    pub fn from_boxed_agent<A: 'a + Agent<R>>(boxed: Box<A>) -> Self {
        return Self {
            data: Box::into_raw(boxed).cast(),
            vtable: const { &DynAgentVTable::new::<A>() },
            _phtm: PhantomData,
        };
    }
}

impl<R> Drop for DynAgent<'_, R> {
    #[inline(always)]
    fn drop(&mut self) {
        (self.vtable.drop)(self.data)
    }
}

pub(crate) struct DynAgentVTable<R> {
    handle: for<'a> fn(
        *mut (),
        &'a mut R,
    ) -> Pin<Box<dyn 'a + Future<Output = color_eyre::Result<()>>>>,
    drop: fn(*mut ()),
}

impl<R: Record> DynAgentVTable<R> {
    pub const fn new<A: Agent<R>>() -> Self {
        fn drop_agent<R: Record, A: Agent<R>>(ptr: *mut ()) {
            unsafe { core::ptr::drop_in_place(ptr.cast::<A>()) }
        }

        fn handle_agent<'a, R: Record, A: Agent<R>>(
            ptr: *mut (),
            record: &'a mut R,
        ) -> Pin<Box<dyn 'a + Future<Output = color_eyre::Result<()>>>> {
            return Box::pin(async move {
                let this = unsafe { &mut *ptr.cast::<A>() };
                return this.handle(record).await.map_err(Into::into);
            });
        }

        return Self {
            handle: handle_agent::<R, A>,
            drop: drop_agent::<R, A>,
        };
    }
}
