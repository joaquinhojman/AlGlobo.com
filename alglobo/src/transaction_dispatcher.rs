use crate::entity_sender::{EntitySender, PrepareTransaction};
use crate::LogMessage;
use crate::Logger;
use actix::{Actor, Addr, Context, Handler, Message};
use alglobo_common_utils::transaction_request::TransactionRequest;

use csv::StringRecord;

const HEADER_ID: &str = "id";
const HEADER_HOTEL: &str = "hotel_cost";
const HEADER_BANK: &str = "bank_cost";
const HEADER_AIRLINE: &str = "airline_cost";

pub struct TransactionDispatcher {
    messenger: Addr<EntitySender>,
    logger: Addr<Logger>,
}

impl TransactionDispatcher {
    pub fn new(messenger: Addr<EntitySender>, logger: Addr<Logger>) -> Self {
        logger.do_send(LogMessage::new(
            "Creating TransactionDispatcher...".to_string(),
        ));
        TransactionDispatcher { messenger, logger }
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

    pub fn deserialize(&self, logger: &Addr<Logger>) -> TransactionRequest {
        let header = StringRecord::from(vec![HEADER_ID, HEADER_HOTEL, HEADER_BANK, HEADER_AIRLINE]);
        let raw_transaction = self.transaction.clone();
        /*logger.do_send(LogMessage::new(format!(
            "deserialize: {:?}",
            raw_transaction
        )));
         */
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
        let transaction = raw_transaction.deserialize(&self.logger);
        let msg = PrepareTransaction::new(transaction);
        self.logger.do_send(LogMessage::new(
            "[DISPATCHER] sending to messenger".to_string(),
        ));
        let _ = self.messenger.do_send(msg);
    }
}
