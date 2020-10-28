use std::{io::Read, marker::PhantomData};

#[cfg(test)]
mod tests;

pub struct Receiver<I, T> {
    inner: I,
    buf: Vec<u8>,
    buf_size: usize,
    _phantom: PhantomData<fn() -> T>,
}

impl<I, T> Receiver<I, T> {
    pub fn new(inner: I, buf_size: usize) -> Self {
        Self {
            inner,
            buf_size,
            buf: Vec::with_capacity(buf_size),
            _phantom: PhantomData,
        }
    }
}

impl<I, T> crate::Receiver<T> for Receiver<I, T> where T: serde::de::DeserializeOwned, I: Read {
    type Error = bincode::Error;

    fn receive(&mut self) -> Result<T, Self::Error> {
        let result = (|| {
            let size = {
                let mut size_buf = [0; 8];
                self.inner.read_exact(&mut size_buf)?;
                u64::from_le_bytes(size_buf) as usize
            };
            self.buf.resize_with(size, u8::default);
            self.inner.read_exact(self.buf.as_mut())?;
            bincode::deserialize(self.buf.as_ref())
        })();
        self.buf.truncate(self.buf_size);
        self.buf.shrink_to_fit();
        result
    }
}
