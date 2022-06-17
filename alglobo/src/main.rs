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
use tracing::info;
use tracing_subscriber::prelude::*;

const LOG_PATH: &str = "logf.log";
const EINVAL_ARGS: &str = "Invalid arguments";

fn main() -> Result<(), ()> {
    tracing_subscriber::registry()
        .with(Logger::new(LOG_PATH.to_string()))
        .init();
    info!(event = "Server up");

    let actor_system = System::new();

    let argv = args().collect::<Vec<String>>();
    if argv.len() != 2 {
        info!(event = EINVAL_ARGS);
        exit(1);
    }

    let file_path = argv[1].clone();

    let mut entity_map = HashMap::new();
    let (sx, tx) = mpsc::channel();
    let sender = Arc::new(Mutex::new(sx));
    for entity_handler in &[Entity::Hotel, Entity::Bank, Entity::Airline] {
        let entity_arbiter = Arbiter::new();
        let addr = "localhost:8888".to_string();
        let local_sender = sender.clone();
        let execution = async move {
            let entity_addr = EntityMessenger::new(*entity_handler, addr).start();
            local_sender.lock().unwrap().send(entity_addr).unwrap();
            println!("hello from {:?}", entity_handler);
        };
        entity_arbiter.spawn(execution);
        let entity_addr = tx.recv().unwrap();
        entity_map.insert(*entity_handler, entity_addr);
    }

    actor_system.block_on(async {
        let transaction_dispatcher = TransactionDispatcher::new(entity_map).start();
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
