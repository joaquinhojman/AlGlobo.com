use crate::LogMessage;
use actix::{Actor, Addr, Context, Handler, Message};
use std::fs::File;

use crate::logger::LoggerActor;
use csv::{StringRecord, Writer};

const HEADER_ID: &str = "id";
const HEADER_HOTEL: &str = "hotel_cost";
const HEADER_BANK: &str = "bank_cost";
const HEADER_AIRLINE: &str = "airline_cost";

pub struct FileWriter {
    transaction_file: Writer<File>,
    logger: Addr<LoggerActor>,
}

impl FileWriter {
    pub fn new(
        transaction_file_path: String,
        logger: Addr<LoggerActor>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        logger.do_send(LogMessage::new("Creating FileWriter...".to_string()));
        Ok(FileWriter {
            transaction_file: Writer::from_path(transaction_file_path)?,
            logger,
        })
    }
}

impl Actor for FileWriter {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        println!("WRITING TO FILE");
        self.transaction_file
            .write_record(&[HEADER_ID, HEADER_HOTEL, HEADER_BANK, HEADER_AIRLINE])
            .expect("could not write record to file");
        self.transaction_file.flush().expect("could not flush");
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
            .transaction_file
            .write_record(msg.raw_transaction.as_byte_record())
        {
            self.logger.do_send(LogMessage::new(format!(
                "Saved failed transaction, with error message: {}",
                what
            )));
        }
    }
}
