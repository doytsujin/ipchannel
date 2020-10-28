use std::{io::Write, marker::PhantomData};

#[cfg(test)]
mod tests;

pub struct Sender<I, T> {
    inner: I,
    buf: Vec<u8>,
    buf_size: usize,
    _phantom: PhantomData<fn(T)>,
}

impl<I, T> Sender<I, T> {
    pub fn new(inner: I, buf_size: usize) -> Self {
        Self {
            inner,
            buf_size,
            buf: Vec::with_capacity(buf_size),
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
        self.buf.truncate(0);
        // Wannabe try block
        let result = (|| {
            bincode::serialize_into(&mut self.buf, &message)?;
            self.inner
                .write_all(&(self.buf.len() as u64).to_le_bytes())?;
            self.inner.write_all(&self.buf)?;
            self.inner.flush()?;
            Ok(())
        })();
        self.buf.truncate(self.buf_size);
        self.buf.shrink_to_fit();
        result.map(|_| ())
    }
}
