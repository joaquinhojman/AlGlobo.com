use crate::LogMessage;
use actix::{Actor, Addr, Context, Handler, Message};
use std::fs::File;
use alglobo_common_utils::transaction_request::TransactionRequest;


use crate::logger::LoggerActor;
use actix::dev::MessageResponse;
use csv::{Writer, StringRecord};

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
}

impl Handler<TransactionRequest> for FileWriter {
    fn handle(&mut self, msg: TransactionRequest, _ctx: &mut Self::Context) {
        self.transaction_file.write_record(&mut msg)
        logger.do_send(LogMessage::new("Saved failed transaction".to_string()));
    }
}
