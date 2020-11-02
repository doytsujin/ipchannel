use super::*;

#[derive(Debug)]
struct Dummy(i32);

impl<T> Sender<T> for Dummy {
    type Error = ();

    fn send(&mut self, _message: T) -> Result<(), Self::Error> {
        panic!("send should not be called")
    }
}

impl<T> Receiver<T> for Dummy {
    type Error = ();

    fn receive(&mut self) -> Result<T, Self::Error> {
        panic!("receive should not be called")
    }
}

#[derive(Debug)]
struct DummyServer(i32, i32);

impl<S, R> Server<S, R> for DummyServer {
    type Sender = Dummy;
    type Receiver = Dummy;
    type Error = ();

    fn ident(&self) -> String {
        panic!("ident should not be called")
    }

    fn accept(&mut self) -> Result<(Self::Sender, Self::Receiver), Self::Error> {
        self.0 += 1;
        if self.0 == self.1 {
            Err(())
        } else {
            Ok((Dummy(self.0), Dummy(self.0)))
        }
    }
}

#[test]
fn test_serve() {
    let mut counter = 0;
    let assert_sr = |s: Dummy, r: Dummy| {
        counter += 1;
        assert_eq!(s.0, counter);
        assert_eq!(r.0, counter);
        if counter == 5 {
            Serving::Stop("enough")
        } else {
            Serving::Continue
        }
    };
    assert_eq!(
        Server::<(), ()>::serve(&mut DummyServer(0, -1), assert_sr),
        Ok("enough")
    );
    assert_eq!(counter, 5);
}

#[test]
fn test_serve_accept_error() {
    let mut counter = 0;
    let assert_sr = |s: Dummy, r: Dummy| {
        counter += 1;
        assert_eq!(s.0, counter);
        assert_eq!(r.0, counter);
        if counter == 6 {
            Serving::Stop("enough")
        } else {
            Serving::Continue
        }
    };
    assert_eq!(
        Server::<(), ()>::serve(&mut DummyServer(0, 5), assert_sr),
        Err(())
    );
    assert_eq!(counter, 4);
}

#[test]
#[ignore]
fn test_par_serve() {
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    let calls: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(Vec::new()));
    let push_sr = |s: Dummy, r: Dummy| {
        std::thread::sleep(Duration::from_secs(1));
        assert_eq!(s.0, r.0);
        let mut calls = dbg!(calls.lock().unwrap());
        calls.push(s.0);
        if calls.len() == 5 {
            Serving::Stop("enough")
        } else {
            Serving::Continue
        }
    };

    let before = Instant::now();
    println!("HERE NOW STARTING PAR SERVE");
    assert_eq!(
        Server::<(), ()>::par_serve(&mut DummyServer(0, -1), 4, push_sr),
        Ok("enough")
    );

    calls.lock().unwrap().sort();
    assert_eq!(calls.lock().unwrap().as_slice(), &[1, 2, 3, 4]);
    assert!(before.elapsed() < Duration::from_secs(2));
}
