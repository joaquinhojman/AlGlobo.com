use crate::transaction_dispatcher::TransactionDispatcher;
use actix::{Actor, Addr, Context, Handler, Message};
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Read};

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

#[derive(Message)]
#[rtype(result = "String")]
pub struct GetNextTransaction {}

impl Handler<GetNextTransaction> for FileReader {
    type Result = String;

    fn handle(&mut self, _msg: GetNextTransaction, _ctx: &mut Self::Context) -> Self::Result {
        let mut line = String::new();
        match self.transaction_file.read_line(&mut line) {
            Ok(_) => line,
            Err(e) => {
                println!("Rust la concha de tu madre {:?}", e);
                format!("{:?}", e)
            }
        }
    }
}
