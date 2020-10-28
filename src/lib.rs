#![allow(dead_code)]

use std::convert::Infallible;

pub mod tcp;
pub mod generic;

mod sender;
pub use sender::Sender;

mod receiver;
pub use receiver::{Receiver, AsIter as ReceiverAsIter};


pub trait Server<S, R> {
    type Error;
    type Sender: Sender<S>;
    type Receiver: Receiver<R>;

    fn ident(&self) -> String;
    fn accept(&mut self) -> Result<(Self::Sender, Self::Receiver), Self::Error>;

    fn serve<F>(&mut self, mut func: F) -> Result<Infallible, Self::Error>
    where
        F: FnMut(Self::Sender, Self::Receiver),
    {
        loop {
            let (tx, rx) = self.accept()?;
            func(tx, rx);
        }
    }

    fn par_serve<F>(&mut self, pool_size: u32, func: F) -> Result<Infallible, Self::Error>
    where
        Self::Sender: Send,
        Self::Receiver: Send,
        F: Fn(Self::Sender, Self::Receiver) + Send + Sync + 'static,
    {
        let mut pool = scoped_threadpool::Pool::new(pool_size);
        pool.scoped(|scope| loop {
            let (tx, rx) = self.accept()?;
            scope.execute(|| func(tx, rx));
        })
    }
}
