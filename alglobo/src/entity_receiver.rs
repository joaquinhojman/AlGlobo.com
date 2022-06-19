use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use alglobo_common_utils::transaction_response::{
    TransactionResponse, TRANSACTION_RESPONSE_PAYLOAD_SIZE,
};
use std::net::UdpSocket;

use crate::transaction_coordinator::{TransactionCoordinator, TransactionUpdate};

pub struct EntityReceiver {
    stream: UdpSocket,
    transaction_coordinator: Addr<TransactionCoordinator>,
}

impl EntityReceiver {
    pub fn new(stream: UdpSocket, transaction_coordinator: Addr<TransactionCoordinator>) -> Self {
        EntityReceiver {
            stream,
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

    fn handle(&mut self, msg: ReceiveEntityResponse, ctx: &mut Self::Context) -> Self::Result {
        let mut buf = [0u8; TRANSACTION_RESPONSE_PAYLOAD_SIZE];
        if let Ok((_, _)) = self.stream.recv_from(&mut buf) {
            let res: TransactionResponse = buf.to_vec().into();
            self.transaction_coordinator
                .do_send(TransactionUpdate::new(res));
        }
        ctx.address().do_send(msg);
    }
}
