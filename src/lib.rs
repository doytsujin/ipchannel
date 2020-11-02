#![allow(dead_code)]

#[cfg(test)]
mod tests;

pub mod generic;
pub mod tcp;

mod sender;
pub use sender::Sender;

mod receiver;
pub use receiver::{AsIter as ReceiverAsIter, Receiver};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Serving<T> {
    Continue,
    Stop(T),
}

pub trait Server<S, R> {
    type Error;
    type Sender: Sender<S>;
    type Receiver: Receiver<R>;

    fn ident(&self) -> String;
    fn accept(&mut self) -> Result<(Self::Sender, Self::Receiver), Self::Error>;

    fn serve<T, F>(&mut self, mut func: F) -> Result<T, Self::Error>
    where
        F: FnMut(Self::Sender, Self::Receiver) -> Serving<T>,
    {
        loop {
            let (tx, rx) = self.accept()?;
            match func(tx, rx) {
                Serving::Stop(res) => return Ok(res),
                Serving::Continue => {}
            }
        }
    }

    fn par_serve<T, F>(&mut self, pool_size: u32, func: F) -> Result<T, Self::Error>
    where
        Self: Send,
        Self::Error: Send,
        Self::Sender: Send,
        Self::Receiver: Send,
        T: Send + std::fmt::Debug,
        F: Fn(Self::Sender, Self::Receiver) -> Serving<T> + Send + Sync,
    {
        use std::sync::mpsc::{self, TryRecvError};

        let (result_tx, result_rx) = mpsc::sync_channel(pool_size as usize);
        let (conn_tx, conn_rx) = mpsc::sync_channel(0);

        crossbeam::thread::scope(|scope| {
            let reader = scope.spawn(move |_| loop {
                match result_rx.try_recv() {
                    Ok(x) => return Ok(x),
                    Err(TryRecvError::Disconnected) => {
                        unreachable!("result_tx should never disconnect")
                    }
                    Err(TryRecvError::Empty) => {}
                };
                conn_tx.send(self.accept()?).expect("conn_rx should never disconnect");
            });

            scoped_threadpool::Pool::new(pool_size).scoped(|scope| {
                while let Ok((tx, rx)) = conn_rx.recv() {
                    scope.execute(|| {
                        if let Serving::Stop(x) = func(tx, rx) {
                            result_tx.send(x).ok();
                        }
                    })
                }
            });

            reader.join().unwrap()
        }).unwrap()
    }
}
