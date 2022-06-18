use alglobo_common_utils::entity_payload::EntityPayload;
use rand::{thread_rng, Rng};
use std::net::UdpSocket;
use std::process::exit;

fn main() {
    println!("Hello, world!");
    let sock: UdpSocket;
    if let Ok(udpsock) = UdpSocket::bind("localhost:1234") {
        sock = udpsock;
    } else {
        exit(1);
    }
    let mut rng = thread_rng();
    loop {
        let mut buf = [0u8; 16];
        println!("awaiting buffer");
        if let Ok((_, addr)) = sock.recv_from(&mut buf) {
            let v = buf.to_vec();
            let payload_deserialized: EntityPayload = v.into();
            println!("Received: {:?} from addr {}", payload_deserialized, addr);
            let x: f64 = rng.gen();
            println!("{}", x);

            let _r = sock.send_to(&buf, addr);
        }
    }
}
