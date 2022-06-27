use crate::transaction_dispatcher::{ReceiveTransaction, TransactionDispatcher};
use crate::file_writer::{FileWriter, FailedTransaction};
use crate::LogMessage;
use actix::{Actor, Addr, Context, Handler, Message};
use std::fs::File;

use crate::logger::LoggerActor;
use actix::dev::MessageResponse;
use csv::{Reader, StringRecord};

pub struct FileReader {
    transaction_file_handle: Reader<File>,
    transaction_dispatcher: Addr<TransactionDispatcher>,
    failed_transaction_logger: Addr<FileWriter>,
    logger: Addr<LoggerActor>,
}

impl FileReader {
    pub fn new(
        transaction_file_path: String,
        transaction_dispatcher: Addr<TransactionDispatcher>,
        failed_transaction_logger: Addr<FileWriter>,
        logger: Addr<LoggerActor>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        logger.do_send(LogMessage::new("Creating FileReader...".to_string()));
        Ok(FileReader {
            transaction_file_handle: Reader::from_path(transaction_file_path)?,
            transaction_dispatcher,
            failed_transaction_logger,
            logger,
        })
    }
}

impl Actor for FileReader {
    type Context = Context<Self>;
}

#[derive(MessageResponse)]
pub enum ReadStatus {
    KeepReading,
    Eof,
    ParseError(csv::Error),
}

#[derive(Message, Clone, Copy)]
#[rtype(result = "ReadStatus")]
pub struct ServeNextTransaction {}

impl Handler<ServeNextTransaction> for FileReader {
    type Result = ReadStatus;

    fn handle(&mut self, _msg: ServeNextTransaction, _ctx: &mut Self::Context) -> Self::Result {
        let mut record = StringRecord::new();
        match self.transaction_file_handle.read_record(&mut record) {
            Ok(any_left) => {
                if any_left {
                    let response = ReceiveTransaction::new(record);
                    self.logger.do_send(LogMessage::new(
                        "FileReader: Sending to transaction_dispatcher".to_string(),
                    ));
                    self.transaction_dispatcher.do_send(response);
                    return ReadStatus::KeepReading;
                }
                ReadStatus::Eof
            }
            Err(e) => ReadStatus::ParseError(e),
        }
    }
}

#[derive(Message, Clone, Copy)]
#[rtype(result = "()")]
pub struct FindTransaction {
    pub transaction_id: u64,
}

impl Handler<FindTransaction> for FileReader {
    type Result = ();

    fn handle(&mut self, msg: FindTransaction, _ctx: &mut Self::Context) -> Self::Result {
        let mut record = StringRecord::new();

        while self.transaction_file_handle.read_record(&mut record).unwrap() {
            let transaction = ReceiveTransaction::new(record).deserialize(&self.logger);
            if transaction.get_transaction_id() == msg.transaction_id {
                self.logger.do_send(LogMessage::new(
                    "FileReader: found specific transaction".to_string(),
                ));
                let mut field = vec![];
                let v = transaction.get_entities_data();
                for (_entity, data) in v {
                    field.push(data);
                }
                let failed = FailedTransaction {
                    id: transaction.get_transaction_id(),
                    hotel_cost: field[1].cost,
                    bank_cost: field[2].cost,
                    airline_cost: field[3].cost,
                };
                self.failed_transaction_logger.do_send(failed);
            }
            record = StringRecord::new();
        }
        self.logger.do_send(LogMessage::new(
            "FileReader: couldnt find specific transaction".to_string(),
        ));
    }
}
