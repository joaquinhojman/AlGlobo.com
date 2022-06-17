use crate::entity_data::Entity;
use crate::entity_messenger::{EntityMessenger, ReceiveEntityTransaction};
use crate::transaction::Transaction;
use actix::{Actor, Addr, Context, Handler, Message};
use csv::StringRecord;
use std::collections::HashMap;

const HEADER_ID: &str = "id";
const HEADER_HOTEL: &str = "hotel_cost";
const HEADER_BANK: &str = "bank_cost";
const HEADER_AIRLINE: &str = "airline_cost";

pub struct TransactionDispatcher {
    entity_mapping: HashMap<Entity, Addr<EntityMessenger>>,
}

impl TransactionDispatcher {
    pub fn new(entity_mapping: HashMap<Entity, Addr<EntityMessenger>>) -> Self {
        TransactionDispatcher { entity_mapping }
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
        let entities_data = transaction.get_entities_data();
        let _ = entities_data
            .into_iter()
            .map(|t| {
                let (entity_type, entity_data) = t;
                match self.entity_mapping.get(&entity_type) {
                    Some(addr) => {
                        let msg = ReceiveEntityTransaction::new(entity_data);
                        addr.do_send(msg);
                    }
                    None => {
                        eprintln!("Invalid transaction arrived");
                    }
                };
            })
            .collect::<Vec<()>>();
    }
}
