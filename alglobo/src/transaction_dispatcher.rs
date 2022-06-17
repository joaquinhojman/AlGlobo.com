use crate::entity_data::Entity;
use crate::entity_messenger::{EntityMessenger, ServeTransaction};
use crate::transaction::Transaction;
use actix::{Actor, Addr, Context, Handler, Message};
use csv::StringRecord;
use std::collections::HashMap;

const HEADER_ID: &str = "id";
const HEADER_HOTEL: &str = "hotel_cost";
const HEADER_BANK: &str = "bank_cost";
const HEADER_AIRLINE: &str = "airline_cost";

pub struct TransactionDispatcher {
    messenger: Addr<EntityMessenger>,
}

impl TransactionDispatcher {
    pub fn new(messenger: Addr<EntityMessenger>) -> Self {
        TransactionDispatcher { messenger }
    }
}

impl Actor for TransactionDispatcher {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ReceiveTransaction {
    transaction: StringRecord,
}

impl ReceiveTransaction {
    pub fn new(transaction: StringRecord) -> Self {
        ReceiveTransaction { transaction }
    }

    pub fn deserialize(&self) -> Transaction {
        let header = StringRecord::from(vec![HEADER_ID, HEADER_HOTEL, HEADER_BANK, HEADER_AIRLINE]);
        let raw_transaction = self.transaction.clone();
        match raw_transaction.deserialize(Some(&header)) {
            Ok(transaction) => transaction,
            Err(e) => {
                eprintln!("{}", e);
                panic!()
            }
        }
    }
}

impl Handler<ReceiveTransaction> for TransactionDispatcher {
    type Result = ();
    fn handle(
        &mut self,
        raw_transaction: ReceiveTransaction,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let transaction = raw_transaction.deserialize();
        let msg = ServeTransaction::new(transaction);
        println!("[DISPATCHER] sending to messenger");
        let _ = self.messenger.send(msg);
    }
}
