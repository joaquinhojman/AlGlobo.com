use crate::statistics_handler::{RegisterTransaction, StatisticsHandler, UnregisterTransaction};
use crate::transaction_coordinator::{TransactionCoordinator, WaitTransactionStateResponse};
use crate::LogMessage;
use crate::Logger;
use actix::{Actor, ActorFutureExt, Context, Handler, Message, ResponseActFuture, WrapFuture};
use actix::{Addr, AsyncContext};
use alglobo_common_utils::entity_type::EntityType;
use alglobo_common_utils::transaction_request::TransactionRequest;
use alglobo_common_utils::transaction_state::TransactionState;
use std::collections::HashMap;
use std::future::Future;
use tokio::net::UdpSocket;
//use std::net::UdpSocket;
use std::sync::Arc;
use std::time::Instant;

/*
   Actor receiver: recibir estado de transaccion (commit, abbort) + id
   -> receiver espera hasta que lleguen 3 commits para esa transaccion
   cuando llegan los 3 commits, se confirma la transaccion (se escribe en el archivo de log)
*/

pub struct EntitySender {
    stream: Arc<UdpSocket>,
    address_map: HashMap<EntityType, String>,
    logger: Addr<Logger>,
    coordinator_addr: Addr<TransactionCoordinator>,
    statistics_handler: Addr<StatisticsHandler>,
    transaction_timestamps: HashMap<u64, Instant>,
}

impl EntitySender {
    pub fn new(
        stream: Arc<UdpSocket>,
        address_map: HashMap<EntityType, String>,
        logger: Addr<Logger>,
        coordinator_addr: Addr<TransactionCoordinator>,
        statistics_handler: Addr<StatisticsHandler>,
    ) -> Self {
        logger.do_send(LogMessage::new("Creating EntitySender...".to_string()));
        EntitySender {
            stream,
            address_map,
            logger,
            coordinator_addr,
            statistics_handler,
            transaction_timestamps: HashMap::new(),
        }
    }

    /*
    fn broadcast_new_transaction(&self, transaction: &TransactionRequest) -> Future<Output=()> + Send + 'static {
        let v = transaction.get_entities_data();
        /*
        self.logger
            .do_send(LogMessage::new(format!("entities_data: {:?}", v)));
        */
        let write_stream = self.stream.clone();
        let addresses = self.address_map.clone();
        async {
            for (entity, data) in v {
                let addr = &addresses[&entity];
                let data_buffer: Vec<u8> = data.into();
                /*self.logger.do_send(LogMessage::new(format!(
                    "[MESSENGER] sending data: {:?}",
                    data_buffer.clone()
                )));
                */
                write_stream.send_to(data_buffer.as_slice(), addr).await.expect(&*format!("{} failed", addr));
            }
        }
    }*/

    /*fn broadcast_state(&self, transaction_id: u64, transaction_state: TransactionState) {
        let temp_bytes = transaction_id.to_be_bytes();
        let send_buffer = temp_bytes.as_slice();
        let state_buffer: u8 = transaction_state.into();
        let mut to_send = Vec::new();
        to_send.push(state_buffer);
        to_send.extend_from_slice(send_buffer);
        drop(
            self.address_map
                .values()
                .map(|x| {
                    self.stream.send_to(to_send.as_slice(), x).unwrap();
                })
                .collect::<Vec<()>>(),
        );
    }*/
}

impl Actor for EntitySender {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PrepareTransaction {
    transaction: TransactionRequest,
}

impl PrepareTransaction {
    pub fn new(transaction: TransactionRequest) -> Self {
        PrepareTransaction { transaction }
    }
}

impl Handler<PrepareTransaction> for EntitySender {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: PrepareTransaction, ctx: &mut Self::Context) -> Self::Result {
        // registramos primero que vamos a esperar a esta transaccion
        // TODO: fix race condition
        let _r = self
            .coordinator_addr
            .do_send(WaitTransactionStateResponse::new(
                msg.transaction.get_transaction_id(),
                TransactionState::Wait,
                TransactionState::Commit,
                ctx.address(),
                true,
            ));
        self.logger.do_send(LogMessage::new(
            "[EntitySender] broadcast_new_transaction".to_string(),
        ));

        let write_stream = self.stream.clone();
        let addresses = self.address_map.clone();
        let v = msg.transaction.get_entities_data();
        let fut = async move {
            for (entity, data) in v {
                let addr = &addresses[&entity];
                let data_buffer: Vec<u8> = data.into();
                write_stream.send_to(data_buffer.as_slice(), addr).await.expect(&*format!("{} failed", addr));
            }
            msg
        };

        Box::pin(fut.into_actor(self).map(|msg, me, ctx| {
            me.transaction_timestamps.insert(msg.transaction.get_transaction_id(), Instant::now());
            me.statistics_handler.do_send(RegisterTransaction::new(
                msg.transaction.get_transaction_id(),
            ));
        }))
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastTransactionState {
    transaction_id: u64,
    transaction_state: TransactionState
}

// este broadcast sirve para Abort o Commited (si se dispara este handler, significa que recibimos
// o un commit o un abort para esa transaccion)
impl BroadcastTransactionState {
    pub fn new(
        transaction_id: u64,
        transaction_state: TransactionState
    ) -> Self {
        BroadcastTransactionState {
            transaction_id,
            transaction_state
        }
    }
}

impl Handler<BroadcastTransactionState> for EntitySender {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: BroadcastTransactionState, ctx: &mut Self::Context) -> Self::Result {
        // si nos llamaron aca, la transaccion ya resolvió su estado (o fue abortada o commiteada)
        // esto es asi porque asumimos que no se puede fallar en la fase de commit (tal cual lo hace el algoritmo)
        let instant = self
            .transaction_timestamps
            .get_mut(&msg.transaction_id)
            .unwrap();
        let duration = instant.elapsed();
        self.statistics_handler
            .do_send(UnregisterTransaction::new(msg.transaction_id, duration));

        let temp_bytes = msg.transaction_id.to_be_bytes();
        let send_buffer = temp_bytes.as_slice();
        let state_buffer: u8 = msg.transaction_state.into();
        let mut to_send = Vec::new();
        to_send.push(state_buffer);
        to_send.extend_from_slice(send_buffer);
        let write_stream = self.stream.clone();
        let addresses = self.address_map.clone();
        let fut = async move {
            for (_, addr) in addresses {
                write_stream.send_to(to_send.as_slice(), addr).await.unwrap();
            }
            msg
        };
        Box::pin(fut.into_actor(self).map(|msg, me, ctx| {
            me.logger.do_send(LogMessage::new(format!(
                "[EntitySender] broadcast_state transaction id: {}",
                msg.transaction_id
            )));
        }))
    }
}
