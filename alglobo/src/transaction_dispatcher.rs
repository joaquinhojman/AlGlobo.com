use crate::entity_sender::{EntitySender, ServeTransaction};
use actix::{Actor, Addr, Context, Handler, Message};
use alglobo_common_utils::transaction_request::TransactionRequest;

use csv::StringRecord;

const HEADER_ID: &str = "id";
const HEADER_HOTEL: &str = "hotel_cost";
const HEADER_BANK: &str = "bank_cost";
const HEADER_AIRLINE: &str = "airline_cost";

pub struct TransactionDispatcher {
    messenger: Addr<EntitySender>,
}

impl TransactionDispatcher {
    pub fn new(messenger: Addr<EntitySender>) -> Self {
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

    pub fn deserialize(&self) -> TransactionRequest {
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
        let _ = self.messenger.do_send(msg);
    }
}
