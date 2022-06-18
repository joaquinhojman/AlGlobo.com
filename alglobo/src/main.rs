extern crate core;
pub use alglobo_common_utils;
mod entity_receiver;
mod entity_sender;
mod file_reader;
mod logger;
mod statistics_handler;
mod transaction_dispatcher;

use crate::logger::Logger;
use file_reader::FileReader;
use std::collections::HashMap;
use transaction_dispatcher::TransactionDispatcher;

use crate::entity_receiver::{EntityReceiver, ReceiveEntityResponse};
use crate::entity_sender::EntitySender;
use crate::file_reader::{ReadStatus, ServeNextTransaction};
use actix::Actor;
use actix_rt::{Arbiter, System};
use alglobo_common_utils::entity_type::EntityType;
use std::env::args;
use std::net::UdpSocket;
use std::process::exit;
use std::ptr::write;
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

    let (sx, tx) = mpsc::channel();

    let addr = "localhost:8888".to_string();
    let write_stream = UdpSocket::bind(addr.clone()).expect("could not bind to addr");
    let read_stream = write_stream.try_clone().unwrap();

    let mut entity_addresses = HashMap::new();

    entity_addresses.insert(EntityType::Hotel, "localhost:1234".to_string());
    entity_addresses.insert(EntityType::Bank, "localhost:1235".to_string());
    entity_addresses.insert(EntityType::Airline, "localhost:1236".to_string());

    let channel_sender = Arc::new(Mutex::new(sx));

    let sender_arbiter = Arbiter::new();
    let receiver_arbiter = Arbiter::new();

    let local_sender = channel_sender.clone();

    // (id, estado)
    let sender_execution = async move {
        let sender_addr = EntitySender::new(write_stream, entity_addresses).start();
        local_sender.lock().unwrap().send(sender_addr).unwrap();
    };

    let receiver_execution = async move {
        let addr = EntityReceiver::new(read_stream).start();
        addr.do_send(ReceiveEntityResponse {});
    };

    receiver_arbiter.spawn(receiver_execution);
    sender_arbiter.spawn(sender_execution);

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
