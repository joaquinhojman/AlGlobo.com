use crate::transaction_coordinator::{TransactionCoordinator, TransactionUpdate};
use crate::LogMessage;
use crate::Logger;
use actix::{ActorFutureExt, ActorStreamExt, Addr, ResponseActFuture, WrapFuture};
use actix::{Actor, AsyncContext, Context, Handler, Message};
use alglobo_common_utils::transaction_response::{
    TransactionResponse, TRANSACTION_RESPONSE_PAYLOAD_SIZE,
};

use tokio::net::UdpSocket;
use std::sync::Arc;

pub struct EntityReceiver {
    stream: Arc<UdpSocket>,
    logger: Addr<Logger>,
    transaction_coordinator: Addr<TransactionCoordinator>,
}

impl EntityReceiver {
    pub fn new(
        stream: Arc<UdpSocket>,
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
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, _msg: ReceiveEntityResponse, ctx: &mut Self::Context) -> Self::Result {
        let mut buf = [0u8; TRANSACTION_RESPONSE_PAYLOAD_SIZE];
        let read_stream = self.stream.clone();

        let fut =  async move {
            if let Ok((_, _)) = read_stream.recv_from(&mut buf).await {
                return Ok(buf.to_vec());
            } else {
                Err(())
            }
        };

        Box::pin(fut.into_actor(self).map(|r, me, ctx | {
            if let Ok(vec) = r {
                me.logger
                    .do_send(LogMessage::new(format!("Recibi: {:?}", vec.as_slice())));
                let res: TransactionResponse = vec.into();
                me.transaction_coordinator
                    .do_send(TransactionUpdate::new(res));
            }
            ctx.address().do_send(ReceiveEntityResponse{});
        }))
    }
}
