use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, Running};
use actix_rt::ArbiterHandle;
use std::net::UdpSocket;

pub struct EntityReceiver {
    stream: UdpSocket,
}

impl EntityReceiver {
    pub fn new(stream: UdpSocket) -> Self {
        EntityReceiver { stream }
    }
}

impl Actor for EntityReceiver {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ReceiveEntityResponse {}

// las respuestas siempre tienen el id de la transaccion y el status (8 + 1 bytes)
impl Handler<ReceiveEntityResponse> for EntityReceiver {
    type Result = ();

    fn handle(&mut self, _msg: ReceiveEntityResponse, ctx: &mut Self::Context) -> Self::Result {
        let mut buf = [0u8; 16];
        if let Ok((_, addr)) = self.stream.recv_from(&mut buf) {
            println!("[De {}] Recibi: {:?}", addr, buf);
        }
        ctx.address().do_send(ReceiveEntityResponse {});
    }
}
