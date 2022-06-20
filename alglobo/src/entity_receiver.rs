use crate::transaction_coordinator::{TransactionCoordinator, TransactionUpdate};
use crate::LogMessage;
use crate::Logger;
use actix::Addr;
use actix::{Actor, AsyncContext, Context, Handler, Message};
use alglobo_common_utils::transaction_response::{
    TransactionResponse, TRANSACTION_RESPONSE_PAYLOAD_SIZE,
};
use std::net::UdpSocket;

pub struct EntityReceiver {
    stream: UdpSocket,
    logger: Addr<Logger>,
    transaction_coordinator: Addr<TransactionCoordinator>,
}

impl EntityReceiver {
    pub fn new(
        stream: UdpSocket,
        logger: Addr<Logger>,
        transaction_coordinator: Addr<TransactionCoordinator>,
    ) -> Self {
        logger.do_send(LogMessage::new("Creating EntityReceiver...".to_string()));
        EntityReceiver {
            stream,
            logger,
            transaction_coordinator,
        }
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
        let mut buf = [0u8; TRANSACTION_RESPONSE_PAYLOAD_SIZE];
        if let Ok((_, addr)) = self.stream.recv_from(&mut buf) {
            let res: TransactionResponse = buf.to_vec().into();
            self.transaction_coordinator
                .do_send(TransactionUpdate::new(res));
            self.logger
                .do_send(LogMessage::new(format!("[De {}] Recibi: {:?}", addr, buf)));
        }
        ctx.address().do_send(ReceiveEntityResponse {});
    }
}
