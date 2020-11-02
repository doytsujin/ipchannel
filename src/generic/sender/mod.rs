use std::{io::Write, marker::PhantomData};

#[cfg(test)]
mod tests;

pub struct Sender<I, T> {
    inner: I,
    _phantom: PhantomData<fn(T)>,
}

impl<I, T> Sender<I, T> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<I, T> crate::Sender<T> for Sender<I, T>
where
    T: serde::Serialize,
    I: Write,
{
    type Error = bincode::Error;

    fn send(&mut self, message: T) -> Result<(), Self::Error> {
        bincode::serialize_into(&mut self.inner, &message)?;
        self.inner.flush()?;
        Ok(())
    }
}
