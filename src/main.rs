use ipchannel::{tcp, Receiver as _, Sender as _, Server as _};

fn main() -> anyhow::Result<()> {
    if let Some(Ok(port)) = std::env::args().nth(1).map(|p| p.parse()) {
        let (mut tx, mut rx) = tcp::connect(("127.0.0.1", port))?;
        println!("Connected");

        tx.send("Foo bar baz")?;
        println!("Sent string");

        let x: String = rx.receive()?;
        println!("Received from server: {}", x);

        tx.send("Lolololol")?;
        println!("Sent another");

        let x: String = rx.receive()?;
        println!("Recieved from server: {}", x);
    } else {
        let mut server = tcp::Server::new()?;
        println!("My addr is {}", server.ident());

        server.par_serve(4, |mut tx, mut rx| {
            println!("Accepted connection");

            loop {
                let x: String = match rx.receive() {
                    Ok(x) => x,
                    Err(err) => match *err {
                        bincode::ErrorKind::Io(err) => match err.kind() {
                            std::io::ErrorKind::UnexpectedEof => break,
                            _ => panic!(err),
                        },
                        _ => panic!(err),
                    }
                };
                println!("Received from client: {}", x);

                let x = x.to_uppercase();
                tx.send(x).unwrap();
            }
        })?;
    }

    Ok(())
}
