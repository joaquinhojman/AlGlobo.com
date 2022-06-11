extern crate core;

mod file_reader;
mod transaction_dispatcher;
mod logger;
use file_reader::FileReader;
use transaction_dispatcher::TransactionDispatcher;
use crate::logger::Logger;

use crate::file_reader::GetNextTransaction;
use actix::Actor;
use actix_rt::System;
use std::env::args;
use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use tracing::info;
use tracing_subscriber::prelude::*;

fn main() -> Result<(), ()> {
    //init logger
    tracing_subscriber::registry()
    .with(Logger::new("logf.log".to_string()))
    .init();
    //log
    info!(event = "Server up");

    let actor_system = System::new();

    let argv = args().collect::<Vec<String>>();
    if argv.len() != 2 {
        println!("Flaco mete bien los argumentos, saludos");
        // logError
        return Err(());
    }

    // actor para lectura de archivos
    let file = &argv[1];
    let file_handle = match File::open(file) {
        Ok(file_handle) => {
            println!("pude abrir XD");
            file_handle
        }
        Err(e) => {
            println!("Error porque se le canto el orto {:?}", e);
            return Err(());
        }
    };

    actor_system.block_on(async {
        let transaction_dispatcher = TransactionDispatcher::new().start();
        let file_reader = FileReader::new(file_handle, transaction_dispatcher).start();
        let msg = GetNextTransaction {};
        let recv1 = file_reader.send(msg).await;
        println!("{:?}", recv1);
        let msg2 = GetNextTransaction {};
        let recv2 = file_reader.send(msg2).await;
        println!("{:?}", recv2);
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
