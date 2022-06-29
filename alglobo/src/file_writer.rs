use crate::LogMessage;
use actix::{Actor, Addr, Context, Handler, Message};
use std::fs::{File, OpenOptions};

use crate::file_reader::DONE_TRANSACTIONS_PATH;
use crate::logger::LoggerActor;
use csv::{StringRecord, Writer};

const HEADER_ID: &str = "id";
const HEADER_HOTEL: &str = "hotel_cost";
const HEADER_BANK: &str = "bank_cost";
const HEADER_AIRLINE: &str = "airline_cost";

pub struct FileWriter {
    failed_transaction_file: Writer<File>,
    done_transaction_file: Writer<File>,
    logger: Addr<LoggerActor>,
}

impl FileWriter {
    pub fn new(
        failed_transaction_file_path: String,
        logger: Addr<LoggerActor>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        logger.do_send(LogMessage::new("Creating FileWriter...".to_string()));
        let done_transaction_file = match OpenOptions::new()
            .write(true)
            .append(true)
            .open(DONE_TRANSACTIONS_PATH)
        {
            Ok(file) => file,
            Err(_) => File::create(DONE_TRANSACTIONS_PATH).unwrap(),
        };

        let mut result = FileWriter {
            failed_transaction_file: Writer::from_path(failed_transaction_file_path)?,
            done_transaction_file: Writer::from_writer(done_transaction_file),
            logger,
        };
        result
            .done_transaction_file
            .write_record(&[HEADER_ID])
            .expect("could not write record to file");
        result
            .done_transaction_file
            .flush()
            .expect("could not flush");
        Ok(result)
    }
}

impl Actor for FileWriter {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        self.failed_transaction_file
            .write_record(&[HEADER_ID, HEADER_HOTEL, HEADER_BANK, HEADER_AIRLINE])
            .expect("could not write record to file");

        self.failed_transaction_file
            .flush()
            .expect("could not flush");
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct FailedTransaction {
    raw_transaction: StringRecord,
}

impl FailedTransaction {
    pub fn new(raw_transaction: StringRecord) -> Self {
        FailedTransaction { raw_transaction }
    }
}

impl Handler<FailedTransaction> for FileWriter {
    type Result = ();

    fn handle(&mut self, msg: FailedTransaction, _ctx: &mut Self::Context) -> Self::Result {
        if let Err(what) = self
            .failed_transaction_file
            .write_record(msg.raw_transaction.as_byte_record())
        {
            self.logger.do_send(LogMessage::new(format!(
                "Saved failed transaction, with error message: {}",
                what
            )));
        } else {
            let _ = self.failed_transaction_file.flush();
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterDoneTransactionId {
    transaction_id: u64,
}

impl RegisterDoneTransactionId {
    pub fn new(transaction_id: u64) -> Self {
        Self { transaction_id }
    }
}

impl Handler<RegisterDoneTransactionId> for FileWriter {
    type Result = ();

    fn handle(&mut self, msg: RegisterDoneTransactionId, _: &mut Self::Context) -> Self::Result {
        if let Err(what) = self
            .done_transaction_file
            .write_record([msg.transaction_id.to_string().as_str()])
        {
            self.logger.do_send(LogMessage::new(format!(
                "Failed to save done transaction id, with error message: {}",
                what
            )));
        } else {
            let _ = self.done_transaction_file.flush();
        }
    }
}
