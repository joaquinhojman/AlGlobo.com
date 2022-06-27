use crate::LogMessage;
use actix::{Actor, Addr, Context, Handler, Message};
use std::fs::File;
use serde::Serialize;

use crate::logger::LoggerActor;
use csv::{Writer};

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

#[derive(Message)]
#[rtype(result = "()")]
pub struct FailedTransaction {
    pub id: u64,
    pub hotel_cost: u64,
    pub bank_cost: u64,
    pub airline_cost: u64,
}
impl FailedTransaction {
    pub fn get_transaction(&self) -> [u64; 4] {
        [self.id, self.hotel_cost, self.bank_cost, self.airline_cost]
    }
}

#[derive(Serialize)]
struct Row {
    id: u64,
    hotel_cost: u64,
    bank_cost: u64,
    airline_cost: u64,
}


impl Handler<FailedTransaction> for FileWriter {
    type Result = ();
    fn handle(&mut self, msg: FailedTransaction, _ctx: &mut Self::Context) -> Self::Result {
        let field = msg.get_transaction();
        // self.transaction_file.serialize(Row {
        //     id: field[0],
        //     hotel_cost: field[1],
        //     bank_cost: field[2],
        //     airline_cost: field[3],
        // }).unwrap();
        let mut test = [
            field[0].to_string(),
            field[1].to_string(),
            field[2].to_string(),
            field[3].to_string()
        ];
// este fue mi ultimo intento, ni idea porque no se escribio
        self.transaction_file.write_record(&mut test);
        self.logger.do_send(LogMessage::new("Saved failed transaction".to_string()));
    }
}
