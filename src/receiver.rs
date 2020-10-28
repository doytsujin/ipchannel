pub trait Receiver<T> {
    type Error;

    fn receive(&mut self) -> Result<T, Self::Error>;

    fn as_iter(&mut self) -> AsIter<Self, T>
    where
        Self: Sized,
    {
        AsIter {
            inner: self,
            last_error: None,
        }
    }
}

#[derive(Debug)]
pub struct AsIter<'a, R, T>
where
    R: Receiver<T>,
{
    inner: &'a mut R,
    pub last_error: Option<R::Error>,
}

impl<'a, R, T> Iterator for AsIter<'a, R, T>
where
    R: Receiver<T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.receive() {
            Ok(item) => Some(item),
            Err(err) => {
                self.last_error = Some(err);
                None
            }
        }
    }
}
