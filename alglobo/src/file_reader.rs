use crate::transaction_dispatcher::{ReceiveTransaction, TransactionDispatcher};
use actix::{Actor, Addr, Context, Handler, Message};
use std::fs::File;

use actix::dev::MessageResponse;
use csv::{Reader, StringRecord};

pub struct FileReader {
    transaction_file_handle: Reader<File>,
    transaction_dispatcher: Addr<TransactionDispatcher>,
}

impl FileReader {
    pub fn new(
        transaction_file_path: String,
        transaction_dispatcher: Addr<TransactionDispatcher>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(FileReader {
            transaction_file_handle: Reader::from_path(transaction_file_path)?,
            transaction_dispatcher,
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
                    self.transaction_dispatcher.do_send(response);
                    return ReadStatus::KeepReading;
                }
                ReadStatus::Eof
            }
            Err(e) => ReadStatus::ParseError(e),
        }
    }
}
