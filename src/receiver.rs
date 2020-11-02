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
    last_error: Option<R::Error>,
}

impl<'a, R, T> AsIter<'a, R, T>
where
    R: Receiver<T>,
{
    fn last_error(&self) -> Option<&R::Error> {
        self.last_error.as_ref()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    struct Dummy(i32, i32);

    impl Receiver<i32> for Dummy {
        type Error = &'static str;

        fn receive(&mut self) -> Result<i32, Self::Error> {
            self.0 += 1;
            if self.0 == self.1 {
                Err("exhausted")
            } else {
                Ok(self.0)
            }
        }
    }

    #[test]
    fn test_as_iter() {
        let mut r = Dummy(0, 5);
        let mut iter = r.as_iter();
        assert_eq!(iter.last_error(), None);
        let res = {
            let mut v = Vec::new();
            v.extend(&mut iter);
            v
        };
        assert_eq!(&res, &[1, 2, 3, 4]);
        assert_eq!(iter.last_error(), Some(&"exhausted"));
    }
}
