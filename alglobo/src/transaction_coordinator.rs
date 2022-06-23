use crate::entity_sender::BroadcastTransactionState;
use crate::EntitySender;
use crate::LogMessage;
use crate::Logger;
use actix::{
    Actor, ActorFutureExt, Addr, Context, Handler, Message, ResponseActFuture, WrapFuture,
};
use alglobo_common_utils::transaction_response::TransactionResponse;
use alglobo_common_utils::transaction_state::TransactionState;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender;
use tokio::time::timeout;

const TIMEOUT_S: u64 = 10;

pub struct TransactionCoordinator {
    transaction_log: HashMap<u64, TransactionState>,
    transaction_update_listening_channels: HashMap<u64, Sender<Vec<Option<TransactionState>>>>,
    entity_states: HashMap<u64, Vec<Option<TransactionState>>>,
    logger: Addr<Logger>,
}

impl TransactionCoordinator {
    pub fn new(logger: Addr<Logger>) -> Self {
        logger.do_send(LogMessage::new(
            "Creating TransactionCoordinator...".to_string(),
        ));
        TransactionCoordinator {
            transaction_log: HashMap::new(),
            transaction_update_listening_channels: HashMap::new(),
            entity_states: HashMap::new(),
            logger,
        }
    }
}

impl Actor for TransactionCoordinator {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct TransactionUpdate {
    transaction_response: TransactionResponse,
}

impl TransactionUpdate {
    pub fn new(transaction_response: TransactionResponse) -> Self {
        TransactionUpdate {
            transaction_response,
        }
    }
}

impl Handler<TransactionUpdate> for TransactionCoordinator {
    type Result = ();

    fn handle(&mut self, msg: TransactionUpdate, _ctx: &mut Self::Context) -> Self::Result {
        let v = match self
            .entity_states
            .get_mut(&msg.transaction_response.transaction_id)
        {
            Some(v) => v,
            None => {
                self.entity_states
                    .insert(msg.transaction_response.transaction_id, Vec::new());
                self.entity_states
                    .get_mut(&msg.transaction_response.transaction_id)
                    .unwrap()
            }
        };
        v.push(Some(msg.transaction_response.transaction_state));
        if v.len() == 3 && (v.iter().all(|opt| opt.is_some())) {
            // unico caso en el que hay que mandar por el canal es cuando estamos esperando
            // ergo, esto solo vale en la fase de Prepare
            if let None | Some(&TransactionState::Wait) = self.transaction_log.get(&msg.transaction_response.transaction_id) {
                self.logger
                    .do_send(LogMessage::new(format!("[COORDINATOR] States for transaction {}: {:?}", msg.transaction_response.transaction_id, v)));
                // si llegue acá mando el estado nuevo de la transaccion
                // este unwrap es correcto ya que sabemos que el vector tiene 3 options con some
                let tx = self
                    .transaction_update_listening_channels
                    .remove(&msg.transaction_response.transaction_id);
                let v = self
                    .entity_states
                    .remove(&msg.transaction_response.transaction_id);
                if let (Some(vec), Some(tx)) = (v, tx)  {
                    // si fallo se droppeo el receiver, con lo cual se llego al timeout, y por ende se aborto la transaccion
                    let _ = tx.send(vec);
                }
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct WaitTransactionStateResponse {
    pub transaction_id: u64,
    pub transaction_state: TransactionState,
    pub expected_transaction_state: TransactionState,
    pub sender_addr: Addr<EntitySender>,
    pub should_await_next_response: bool,
}

impl WaitTransactionStateResponse {
    pub fn new(
        transaction_id: u64,
        transaction_state: TransactionState,
        expected_transaction_state: TransactionState,
        sender_addr: Addr<EntitySender>,
        should_await_next_response: bool,
    ) -> Self {
        WaitTransactionStateResponse {
            transaction_id,
            transaction_state,
            expected_transaction_state,
            sender_addr,
            should_await_next_response,
        }
    }
}

impl Handler<WaitTransactionStateResponse> for TransactionCoordinator {
    type Result = ResponseActFuture<Self, ()>;

    // Non-blocking wait: el coordinador puede manejar otras transacciones concurrentemente
    // ya que el timeout es pollable (o task-based) y por ende no bloqueante
    // el state que recibe es el que debería haber llegado
    // si es otro se manda abort al sender
    fn handle(
        &mut self,
        msg: WaitTransactionStateResponse,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        // si la contenia, entonces ya registramos esta transaccion
        // con lo cual no hace falta esperar a timeout (asumiendo que no se falla en la fase de commit)
        let log_clone = self.logger.clone();
        if self.transaction_log.contains_key(&msg.transaction_id) {
            Box::pin(std::future::ready(()).into_actor(self))
        } else {
            let (tx, rx) = oneshot::channel();
            self.transaction_log
                .insert(msg.transaction_id, msg.transaction_state);
            self.transaction_update_listening_channels
                .insert(msg.transaction_id, tx);
            let fut = async move {
                match timeout(Duration::from_secs(TIMEOUT_S), rx).await {
                    Ok(x) => {
                        if let Ok(mut v) = x {
                            let all_states_match = v.iter().all(|opt| {
                                // no es bonito pero funciona
                                std::mem::discriminant(&msg.expected_transaction_state)
                                    == std::mem::discriminant(&opt.as_ref().unwrap())
                            });
                            if all_states_match {
                                let state = v.remove(0).unwrap();
                                // self.transaction_log.insert(msg.transaction_id, state);
                                msg.sender_addr.do_send(BroadcastTransactionState::new(
                                    msg.transaction_id,
                                    state
                                ));
                                (msg.transaction_id, state)
                            } else {
                                msg.sender_addr.do_send(BroadcastTransactionState::new(
                                    msg.transaction_id,
                                    TransactionState::Abort
                                ));
                                (msg.transaction_id, TransactionState::Abort)
                            }
                        } else {
                            msg.sender_addr.do_send(BroadcastTransactionState::new(
                                msg.transaction_id,
                                TransactionState::Abort
                            ));
                            (msg.transaction_id, TransactionState::Abort)
                        }
                    }
                    Err(_) => {
                        log_clone.do_send(LogMessage::new(format!("[COORDINATOR] Timeout reached for transaction {}", msg.transaction_id)));
                        msg.sender_addr.do_send(BroadcastTransactionState::new(
                            msg.transaction_id,
                            TransactionState::Abort
                        ));
                        (msg.transaction_id, TransactionState::Abort)
                    }
                }
            };
            Box::pin(fut.into_actor(self).map(|(id, state), me, _| {
                me.logger.do_send(LogMessage::new(format!("[COORDINATOR] transaction {} final state: {:?}", id, state.clone())));
                me.transaction_log.insert(id, state);
            }))
        }
    }
}
