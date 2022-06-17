extern crate core;
mod entity_data;
mod entity_messenger;
mod file_reader;
mod logger;
mod statistics_handler;
mod transaction;
mod transaction_dispatcher;

use crate::logger::Logger;
use file_reader::FileReader;
use std::collections::HashMap;
use transaction_dispatcher::TransactionDispatcher;

use crate::entity_data::Entity;
use crate::entity_messenger::EntityMessenger;
use crate::file_reader::{ReadStatus, ServeNextTransaction};
use actix::Actor;
use actix_rt::{Arbiter, System};
use std::env::args;
use std::process::exit;
use std::sync::{mpsc, Arc, Mutex};

const EINVAL_ARGS: &str = "Invalid arguments";


fn main() -> Result<(), ()> {
    
    let actor_system = System::new();
    
    let mut logger = Logger::new("logf.log");

    logger.log("funciona".to_string());
    let argv = args().collect::<Vec<String>>();
    if argv.len() != 2 {
        exit(1);
    }

    let file_path = argv[1].clone();

    let (sx, tx) = mpsc::channel();

    let addr = "localhost:8888".to_string();

    let mut entity_addresses = HashMap::new();

    entity_addresses.insert(Entity::Hotel, "localhost:1234".to_string());
    entity_addresses.insert(Entity::Bank, "localhost:1235".to_string());
    entity_addresses.insert(Entity::Airline, "localhost:1236".to_string());

    let sender = Arc::new(Mutex::new(sx));

    let entity_arbiter = Arbiter::new();
    let local_sender = sender.clone();

    let execution = async move {
        let entity_addr = EntityMessenger::new(addr, entity_addresses).start();
        local_sender.lock().unwrap().send(entity_addr).unwrap();
    };

    entity_arbiter.spawn(execution);
    let entity_addr = tx.recv().unwrap();

    actor_system.block_on(async {
        let transaction_dispatcher = TransactionDispatcher::new(entity_addr).start();
        let file_reader = match FileReader::new(file_path, transaction_dispatcher) {
            Ok(file_reader) => file_reader,
            Err(e) => {
                eprintln!("{}", e);
                panic!();
            }
        }
        .start();

        // esta logica no se donde debería ir
        let msg = ServeNextTransaction {};
        while let Ok(res) = file_reader.send(msg).await {
            match res {
                ReadStatus::KeepReading => {}
                ReadStatus::Eof => {
                    break;
                }
                ReadStatus::ParseError(e) => {
                    eprintln!("{}", e);
                    panic!()
                }
            }
        }
    });

    match actor_system.run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            return Err(());
        }
    }
    Ok(())
}
