use std::collections::HashSet;
use crate::entity_sender::{EntitySender, PrepareTransaction};
use crate::LogMessage;
use actix::{Actor, Addr, Context, Handler, Message};
use alglobo_common_utils::transaction_request::TransactionRequest;

use crate::logger::LoggerActor;
use csv::StringRecord;

const HEADER_ID: &str = "id";
const HEADER_HOTEL: &str = "hotel_cost";
const HEADER_BANK: &str = "bank_cost";
const HEADER_AIRLINE: &str = "airline_cost";

pub struct TransactionDispatcher {
    messenger: Addr<EntitySender>,
    logger: Addr<LoggerActor>,
    done_transactions: HashSet<u64>
}

impl TransactionDispatcher {
    pub fn new(messenger: Addr<EntitySender>, logger: Addr<LoggerActor>) -> Self {
        logger.do_send(LogMessage::new(
            "Creating TransactionDispatcher...".to_string(),
        ));
        TransactionDispatcher { messenger, logger, done_transactions: HashSet::new() }
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

    pub fn deserialize(&self, _: &Addr<LoggerActor>) -> TransactionRequest {
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
        let transaction = raw_transaction.deserialize(&self.logger);
        // if transaction has not already been done, we go ahead and commit
        if !self.done_transactions.contains(&transaction.get_transaction_id()) {
            let msg = PrepareTransaction::new(transaction);
            let _ = self.messenger.do_send(msg);
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SaveDoneTransactions {
    transactions: HashSet<u64>,
}

impl SaveDoneTransactions {
    pub fn new(transactions: HashSet<u64>) -> Self {
        Self { transactions }
    }
}

impl Handler<SaveDoneTransactions> for TransactionDispatcher {
    type Result = ();

    fn handle(&mut self, msg: SaveDoneTransactions, _: &mut Self::Context) -> Self::Result {
        self.done_transactions = msg.transactions;
    }
}
