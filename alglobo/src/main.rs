extern crate core;

mod entity_data;
mod entity_messenger;
mod file_reader;
mod logger;
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
use actix_rt::System;
use std::env::args;
use std::process::exit;
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

    actor_system.block_on(async {
        let entity_map = vec![Entity::Hotel, Entity::Bank, Entity::Airline]
            .into_iter()
            .fold(HashMap::new(), |mut acc, entity| {
                let entity_addr = EntityMessenger::new(entity).start();
                acc.insert(entity, entity_addr);
                acc
            });

        let transaction_dispatcher = TransactionDispatcher::new(entity_map).start();
        let file_reader = match FileReader::new(file_path, transaction_dispatcher) {
            Ok(file_reader) => file_reader,
            Err(e) => {
                eprintln!("{}", e);
                panic!();
            }
        }
        .start();

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
        System::current().stop();
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
