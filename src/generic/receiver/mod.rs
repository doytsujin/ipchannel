use std::{io::Read, marker::PhantomData};

#[cfg(test)]
mod tests;

pub struct Receiver<I, T> {
    inner: I,
    _phantom: PhantomData<fn() -> T>,
}

impl<I, T> Receiver<I, T> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<I, T> crate::Receiver<T> for Receiver<I, T> where T: serde::de::DeserializeOwned, I: Read {
    type Error = bincode::Error;

    fn receive(&mut self) -> Result<T, Self::Error> {
        bincode::deserialize_from(&mut self.inner)
    }
}
