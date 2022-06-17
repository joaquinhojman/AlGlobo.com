use std::io::Read;
use std::net::{TcpListener, TcpStream, UdpSocket};

fn main() {
    println!("Hello, world!");
    let sock = UdpSocket::bind("localhost:1234").unwrap();
    loop {
        let mut buf = [0u8; 16];
        println!("awaiting buffer");
        let (_, _) = sock.recv_from(&mut buf).unwrap();
        println!("{:?}", buf);
    }
}
