use std::{io, marker::PhantomData, net};

pub type Receiver<T> = crate::generic::Receiver<std::net::TcpStream, T>;
pub type Sender<T> = crate::generic::Sender<std::net::TcpStream, T>;

fn split_stream<S, R>(stream: net::TcpStream) -> io::Result<(Sender<S>, Receiver<R>)> {
    let sender = Sender::new(stream.try_clone()?);
    let receiver = Receiver::new(stream);
    Ok((sender, receiver))
}

pub struct Server<S, R> {
    inner: net::TcpListener,
    local_addr: net::SocketAddr,
    _phantom: PhantomData<fn(R) -> S>,
}

impl<S, R> Server<S, R> {
    pub fn new() -> io::Result<Self> {
        let inner = net::TcpListener::bind(("127.0.0.1", 0))?;
        let local_addr = inner.local_addr()?;
        Ok(Self {
            inner,
            local_addr,
            _phantom: PhantomData,
        })
    }
}

impl<S, R> super::Server<S, R> for Server<S, R>
where
    S: serde::Serialize,
    R: serde::de::DeserializeOwned,
{
    type Error = io::Error;
    type Sender = Sender<S>;
    type Receiver = Receiver<R>;

    fn ident(&self) -> String {
        self.local_addr.to_string()
    }

    fn accept(&mut self) -> Result<(Self::Sender, Self::Receiver), Self::Error> {
        let (stream, _addr) = self.inner.accept()?;
        split_stream(stream)
    }
}

pub fn connect<S, R>(ident: impl net::ToSocketAddrs) -> io::Result<(Sender<S>, Receiver<R>)> {
    net::TcpStream::connect(ident).and_then(split_stream)
}
