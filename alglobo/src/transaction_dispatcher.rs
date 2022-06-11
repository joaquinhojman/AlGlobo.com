use crate::entity_messenger::{EntityMessenger, ReceiveEntityTransaction};
use actix::{Actor, Addr, Context, Handler, Message};
use std::process::exit;
use tracing::info;

pub struct TransactionDispatcher {
    entities_mailboxes: Vec<Addr<EntityMessenger>>,
}

impl TransactionDispatcher {
    pub fn new(entities_mailboxes: Vec<Addr<EntityMessenger>>) -> Self {
        TransactionDispatcher { entities_mailboxes }
    }
}

impl Actor for TransactionDispatcher {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ReceiveTransaction {
    transaction: String,
}

impl ReceiveTransaction {
    pub fn new(transaction: String) -> Self {
        ReceiveTransaction { transaction }
    }

    pub fn transform(&self) -> Vec<i64> {
        let mut raw_transaction = self.transaction.clone();
        let _ = raw_transaction.pop();
        let _ = raw_transaction.pop();
        raw_transaction
            .split(';')
            .map(|x| match x.parse::<i64>() {
                Ok(x) => x,
                Err(e) => {
                    info!(
                        event = "Error parsing f64",
                        value = x,
                        error = &format!("{:?}", e)[..]
                    );
                    exit(1);
                }
            })
            .collect::<Vec<i64>>()
    }
}

impl Handler<ReceiveTransaction> for TransactionDispatcher {
    type Result = ();

    fn handle(&mut self, msg: ReceiveTransaction, _ctx: &mut Self::Context) -> Self::Result {
        let msgs = msg.transform();
        for i in 1..4 {
            let mensajito = msgs[i];
            let mensajote = ReceiveEntityTransaction::new(msgs[0], mensajito);
            self.entities_mailboxes[i - 1].do_send(mensajote);
        }
    }
}
