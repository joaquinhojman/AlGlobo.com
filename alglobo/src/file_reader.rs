use crate::transaction_dispatcher::{ReceiveTransaction, TransactionDispatcher};
use actix::{Actor, Addr, Context, Handler, Message};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct FileReader {
    transaction_file: BufReader<File>,
    transaction_dispatcher: Addr<TransactionDispatcher>,
}

impl FileReader {
    pub fn new(
        transaction_file: File,
        transaction_dispatcher: Addr<TransactionDispatcher>,
    ) -> Self {
        FileReader {
            transaction_file: BufReader::new(transaction_file),
            transaction_dispatcher,
        }
    }
}

impl Actor for FileReader {
    type Context = Context<Self>;
}

#[derive(Message, Clone, Copy)]
#[rtype(result = "bool")]
pub struct ServeNextTransaction {}

impl Handler<ServeNextTransaction> for FileReader {
    type Result = bool;

    fn handle(&mut self, _msg: ServeNextTransaction, _ctx: &mut Self::Context) -> Self::Result {
        let mut line = String::new();
        match self.transaction_file.read_line(&mut line) {
            Ok(n) => {
                if n > 0 {
                    let response = ReceiveTransaction::new(line);
                    self.transaction_dispatcher.do_send(response);
                    true
                } else {
                    false
                }
            }
            Err(e) => {
                let response = ReceiveTransaction::new("".to_string());
                self.transaction_dispatcher.do_send(response);
                false
            }
        }
    }
}
