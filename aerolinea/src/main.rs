use std::net::UdpSocket;
use std::process::exit;

fn main() {
    println!("Hello, world!");
    let sock: UdpSocket;
    if let Ok(udpsock) = UdpSocket::bind("localhost:1236") {
        sock = udpsock;
    } else {
        exit(1);
    }
    loop {
        let mut buf = [0u8; 16];
        println!("awaiting buffer");
        let _r = sock.recv_from(&mut buf);
        println!("{:?}", buf);
    }
}
