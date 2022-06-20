use crate::statistics_handler::{RegisterTransaction, StatisticsHandler, UnregisterTransaction};
use crate::transaction_coordinator::{TransactionCoordinator, WaitTransactionStateResponse};
use crate::LogMessage;
use crate::Logger;
use actix::{Actor, Context, Handler, Message};
use actix::{Addr, AsyncContext};
use alglobo_common_utils::entity_type::EntityType;
use alglobo_common_utils::transaction_request::TransactionRequest;
use alglobo_common_utils::transaction_state::TransactionState;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::Instant;

/*
   Actor receiver: recibir estado de transaccion (commit, abbort) + id
   -> receiver espera hasta que lleguen 3 commits para esa transaccion
   cuando llegan los 3 commits, se confirma la transaccion (se escribe en el archivo de log)
*/

pub struct EntitySender {
    stream: UdpSocket,
    address_map: HashMap<EntityType, String>,
    logger: Addr<Logger>,
    coordinator_addr: Addr<TransactionCoordinator>,
    statistics_handler: Addr<StatisticsHandler>,
    transaction_timestamps: HashMap<u64, Instant>,
}

impl EntitySender {
    pub fn new(
        stream: UdpSocket,
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

    fn broadcast_new_transaction(&self, transaction: &TransactionRequest) {
        let v = transaction.get_entities_data();
        for (entity, data) in v {
            let addr = &self.address_map[&entity];
            let data_buffer: Vec<u8> = data.into();
            self.stream
                .send_to(data_buffer.as_slice(), addr)
                .expect("falle XD");
        }
    }

    fn broadcast_state(&self, transaction_id: u64, transaction_state: TransactionState) {
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
    }
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
    type Result = ();

    fn handle(&mut self, msg: PrepareTransaction, ctx: &mut Self::Context) -> Self::Result {
        // registramos primero que vamos a esperar a esta transaccion
        // TODO: fix race condition
        let r = self
            .coordinator_addr
            .do_send(WaitTransactionStateResponse::new(
                msg.transaction.get_transaction_id(),
                TransactionState::Wait,
                TransactionState::Commit,
                ctx.address(),
                true,
            ));
        self.broadcast_new_transaction(&msg.transaction);
        self.transaction_timestamps
            .insert(msg.transaction.get_transaction_id(), Instant::now());
        self.statistics_handler.do_send(RegisterTransaction::new(
            msg.transaction.get_transaction_id(),
        ));
        // TODO: ver como loggear
        /*self.logger
            .do_send(LogMessage::new(format!("entities_data: {:?}", v)));
        self.logger.do_send(LogMessage::new(format!(
            "[MESSENGER] sending data: {:?}",
            data_buffer.clone()
         */
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastTransactionState {
    transaction_id: u64,
    transaction_state: TransactionState,
    should_await_next_response: bool,
}

impl BroadcastTransactionState {
    pub fn new(
        transaction_id: u64,
        transaction_state: TransactionState,
        should_await_next_response: bool,
    ) -> Self {
        BroadcastTransactionState {
            transaction_id,
            transaction_state,
            should_await_next_response,
        }
    }
}

impl Handler<BroadcastTransactionState> for EntitySender {
    type Result = ();

    fn handle(&mut self, msg: BroadcastTransactionState, ctx: &mut Self::Context) -> Self::Result {
        // si no esperamos a la proxima respuesta, significa que ya esta resuelta (commited o aborted)
        // esto es asi porque asumimos que no se puede fallar en la fase de commit (tal cual lo hace el algoritmo)
        if msg.should_await_next_response {
            self.coordinator_addr
                .do_send(WaitTransactionStateResponse::new(
                    msg.transaction_id,
                    msg.transaction_state,
                    msg.transaction_state,
                    ctx.address(),
                    false,
                ));
            let instant = self
                .transaction_timestamps
                .get_mut(&msg.transaction_id)
                .unwrap();
            let duration = instant.elapsed();
            self.statistics_handler
                .do_send(UnregisterTransaction::new(msg.transaction_id, duration))
        }
        self.broadcast_state(msg.transaction_id, msg.transaction_state)
    }
}
