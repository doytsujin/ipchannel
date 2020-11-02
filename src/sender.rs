pub trait Sender<T> {
    type Error;

    fn send(&mut self, message: T) -> Result<(), Self::Error>;
}
