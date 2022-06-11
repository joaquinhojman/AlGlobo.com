extern crate core;

mod entity_messenger;
#[allow(dead_code)]
mod file_reader;
mod logger;
mod transaction_dispatcher;

use crate::logger::Logger;
use file_reader::FileReader;
use transaction_dispatcher::TransactionDispatcher;

use crate::entity_messenger::EntityMessenger;
use crate::file_reader::ServeNextTransaction;
use actix::{Actor, Addr};
use actix_rt::System;
use std::env::args;
use std::fs::File;
use std::process::exit;
use tracing::info;
use tracing_subscriber::prelude::*;

const LOG_PATH: &str = "logf.log";
const EINVAL_ARGS: &str = "Invalid arguments";
const EOPEN_FILE: &str = "Error opening file";

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

    let file_handle = match File::open(&argv[1]) {
        Ok(file_handle) => file_handle,
        Err(e) => {
            info!(event = EOPEN_FILE, error = &format!("{:?}", e)[..]);
            exit(1);
        }
    };

    actor_system.block_on(async {
        // 0: hotel 1: banco 2: aerolinea
        let entity_vec = (0..3)
            .map(|i| EntityMessenger::new(i).start())
            .collect::<Vec<Addr<EntityMessenger>>>();

        let transaction_dispatcher = TransactionDispatcher::new(entity_vec).start();
        let file_reader = FileReader::new(file_handle, transaction_dispatcher).start();
        let msg = ServeNextTransaction {};
        while let Ok(response) = file_reader.send(msg).await {
            if !response {
                break;
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
