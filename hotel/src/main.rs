use std::io::Read;
use std::net::{TcpListener, TcpStream};

fn main() {
    println!("Hello, world!");
    let listener = TcpListener::bind("localhost:8888").unwrap();
    let mut socks = (0..3)
        .map(|_| {
            let (s, _) = listener.accept().expect("Error aceptando");
            s
        })
        .collect::<Vec<TcpStream>>();
    loop {
        let mut buf = [0u8; 16];
        for sock in &mut socks {
            if sock.read_exact(&mut buf).is_ok() {
                println!("{:?}", buf)
            };
        }
    }
}
