use crate::record::Record;
use libopenai::chat::Role;
use std::{convert::Infallible, future::Future, marker::PhantomData, pin::Pin};

/// TODO
pub trait Agent {
    type Error: 'static + std::error::Error + Send + Sync;

    async fn handle(&mut self, record: &mut Record) -> Result<(), Self::Error>;
}

/// An agent that can be executed through shared reference
pub trait AgentRef: Agent {
    async fn handle_ref(&self, record: &mut Record) -> Result<(), Self::Error>;
}

/* BLANKET IMPLS */
impl<A: Agent> Agent for &mut A {
    type Error = A::Error;

    async fn handle(&mut self, record: &mut Record) -> Result<(), Self::Error> {
        A::handle(self, record).await
    }
}

impl<A: AgentRef> Agent for &A {
    type Error = A::Error;

    async fn handle(&mut self, record: &mut Record) -> Result<(), Self::Error> {
        A::handle_ref(self, record).await
    }
}

/* DEFAULT IMPLS */

/// `String`s can be directly used as a [`User`](Role::User) message
impl Agent for String {
    type Error = Infallible;

    async fn handle(&mut self, record: &mut Record) -> Result<(), Self::Error> {
        record.push(Role::User, self.clone());
        return Ok(());
    }
}

impl AgentRef for String {
    async fn handle_ref(&self, record: &mut Record) -> Result<(), Self::Error> {
        record.push(Role::User, self.clone());
        return Ok(());
    }
}

/* DYNAMIC AGENT */

pub(crate) struct DynAgent<'a> {
    data: *mut (),
    vtable: &'static DynAgentVTable,
    _phtm: PhantomData<&'a mut &'a ()>,
}

impl<'a> DynAgent<'a> {
    pub fn handle<'b>(
        &'b mut self,
        record: &'b mut Record,
    ) -> Pin<Box<dyn 'b + Future<Output = color_eyre::Result<()>>>> {
        return (self.vtable.handle)(self.data, record);
    }
}

impl<'a> DynAgent<'a> {
    pub fn from_boxed_agent<A: 'a + Agent>(boxed: Box<A>) -> Self {
        return Self {
            data: Box::into_raw(boxed).cast(),
            vtable: const { &DynAgentVTable::new::<A>() },
            _phtm: PhantomData,
        };
    }
}

impl<'a, A: 'a + Agent> From<A> for DynAgent<'a> {
    #[inline]
    fn from(value: A) -> Self {
        return Self::from_boxed_agent(Box::new(value));
    }
}

impl Drop for DynAgent<'_> {
    #[inline(always)]
    fn drop(&mut self) {
        (self.vtable.drop)(self.data)
    }
}

pub(crate) struct DynAgentVTable {
    handle: for<'a> fn(
        *mut (),
        &'a mut Record,
    ) -> Pin<Box<dyn 'a + Future<Output = color_eyre::Result<()>>>>,
    drop: fn(*mut ()),
}

impl DynAgentVTable {
    pub const fn new<A: Agent>() -> Self {
        fn drop_agent<A: Agent>(ptr: *mut ()) {
            unsafe { core::ptr::drop_in_place(ptr.cast::<A>()) }
        }

        fn handle_agent<'a, A: Agent>(
            ptr: *mut (),
            record: &'a mut Record,
        ) -> Pin<Box<dyn 'a + Future<Output = color_eyre::Result<()>>>> {
            return Box::pin(async move {
                let this = unsafe { &mut *ptr.cast::<A>() };
                return this.handle(record).await.map_err(color_eyre::Report::from);
            });
        }

        return Self {
            handle: handle_agent::<A>,
            drop: drop_agent::<A>,
        };
    }
}
