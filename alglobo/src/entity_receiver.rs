use crate::transaction_coordinator::{TransactionCoordinator, TransactionUpdate};
use crate::LogMessage;
use actix::{Actor, AsyncContext, Context, Handler, Message};
use actix::{ActorFutureExt, Addr, ResponseActFuture, WrapFuture};
use alglobo_common_utils::transaction_response::{
    TransactionResponse, TRANSACTION_RESPONSE_PAYLOAD_SIZE,
};

use crate::logger::LoggerActor;
use std::sync::Arc;
use tokio::net::UdpSocket;

pub struct EntityReceiver {
    stream: Arc<UdpSocket>,
    logger: Addr<LoggerActor>,
    transaction_coordinator: Addr<TransactionCoordinator>,
}

impl EntityReceiver {
    pub fn new(
        stream: Arc<UdpSocket>,
        logger: Addr<LoggerActor>,
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

    fn handle(&mut self, _msg: ReceiveEntityResponse, _: &mut Self::Context) -> Self::Result {
        let mut buf = [0u8; TRANSACTION_RESPONSE_PAYLOAD_SIZE];
        let read_stream = self.stream.clone();

        let fut = async move {
            if let Ok((_, _)) = read_stream.recv_from(&mut buf).await {
                Ok(buf.to_vec())
            } else {
                Err(())
            }
        };

        Box::pin(fut.into_actor(self).map(|r, me, ctx| {
            if let Ok(vec) = r {
                me.logger
                    .do_send(LogMessage::new(format!("Recibi: {:?}", vec.as_slice())));
                let res: TransactionResponse = vec.into();
                me.transaction_coordinator
                    .do_send(TransactionUpdate::new(res));
            }
            ctx.address().do_send(ReceiveEntityResponse {});
        }))
    }
}
