use crate::LogMessage;
use crate::Logger;
use actix::Addr;
use actix::{Actor, AsyncContext, Context, Handler, Message};
use std::net::UdpSocket;

pub struct EntityReceiver {
    stream: UdpSocket,
    logger: Addr<Logger>,
}

impl EntityReceiver {
    pub fn new(stream: UdpSocket, logger: Addr<Logger>) -> Self {
        logger.do_send(LogMessage::new("Creating EntityReceiver...".to_string()));
        EntityReceiver { stream, logger }
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
            self.logger
                .do_send(LogMessage::new(format!("[De {}] Recibi: {:?}", addr, buf)));
            println!("[De {}] Recibi: {:?}", addr, buf);
        }
        ctx.address().do_send(ReceiveEntityResponse {});
    }
}
