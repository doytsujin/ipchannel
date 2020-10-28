use std::{error::Error, fmt::Debug};

#[derive(Debug, thiserror::Error)]
#[error("Failed sending {sent_count} message: {first_error}")]
pub struct SendManyError<E: Debug + Error + 'static> {
    pub sent_count: usize,
    #[source]
    pub first_error: E,
}

pub trait Sender<T> {
    type Error;

    fn send(&mut self, message: T) -> Result<(), Self::Error>;

    fn send_many(
        &mut self,
        messages: impl IntoIterator<Item = T>,
    ) -> Result<(), SendManyError<Self::Error>>
    where
        Self::Error: Debug + Error + 'static,
    {
        for (idx, message) in messages.into_iter().enumerate() {
            if let Err(err) = self.send(message) {
                return Err(SendManyError {
                    sent_count: idx,
                    first_error: err,
                });
            }
        }
        Ok(())
    }
}
