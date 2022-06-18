use std::net::UdpSocket;
use alglobo_common_utils::entity_payload::EntityPayload;
use rand::{Rng, thread_rng};


fn main() {
    println!("Hello, world!");
    let sock = UdpSocket::bind("localhost:1234").unwrap();
    let mut rng = thread_rng();
    loop {
        let mut buf = [0u8; 16];
        println!("awaiting buffer");
        let (_, addr) = sock.recv_from(&mut buf).unwrap();
        let v = buf.to_vec();
        let payload_deserialized: EntityPayload = v.into();
        println!("Received: {:?} from addr {}", payload_deserialized, addr);
        let x: f64 = rng.gen();
        println!("{}", x);

        sock.send_to(&buf, addr).unwrap();
    }
}
