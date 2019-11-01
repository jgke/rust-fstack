use gotham::error::Result;
use gotham::handler::{NewHandler, Handler, IntoHandlerFuture, HandlerFuture};
use gotham::state::State;
use std::panic::RefUnwindSafe;

use crate::db::{DB_CONNECTION, Connection};

#[derive(Copy, Clone, Debug)]
pub struct DBHandlerI<F, R>
where F: FnOnce(State, Connection) -> R + Send,
      R: IntoHandlerFuture {
    f: F
}

#[derive(Debug)]
pub struct DBHandler<F, R>
where F: FnOnce(State, Connection) -> R + Send,
      R: IntoHandlerFuture {
    f: F
}

impl<F, R> NewHandler for DBHandlerI<F, R>
where F: FnOnce(State, Connection) -> R + Copy + Send + Sync + RefUnwindSafe,
      R: IntoHandlerFuture {
    type Instance = DBHandler<F, R>;

    fn new_handler(&self) -> Result<Self::Instance> {
        Ok(DBHandler { f: self.f })
    }
}

impl<F, R> Handler for DBHandler<F, R>
where
F: FnOnce(State, Connection) -> R + Send,
R: IntoHandlerFuture {
    fn handle(self, state: State) -> Box<HandlerFuture> {
        let conn = DB_CONNECTION.take().get().unwrap();
        (self.f)(state, Connection::new(&conn)).into_handler_future()
    }
}

pub fn r<F, R>(f: F) -> DBHandlerI<F, R>
where F: FnOnce(State, Connection) -> R + Send,
      R: IntoHandlerFuture {
    DBHandlerI { f }
}
