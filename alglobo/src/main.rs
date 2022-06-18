extern crate core;
pub use alglobo_common_utils;
mod entity_receiver;
mod entity_sender;
mod file_reader;
mod logger;
mod statistics_handler;
mod transaction_dispatcher;

use crate::logger::Logger;
use crate::logger::LogMessage;
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
use std::sync::{mpsc, Arc, Mutex};

//const EINVAL_ARGS: &str = "Invalid arguments";

fn main() -> Result<(), ()> {
    
    let actor_system = System::new();
    
    //Inicializacion del Actor Logger
    let (sx_l, tx_l) = mpsc::channel();
    let logger_sender = Arc::new(Mutex::new(sx_l));
    let logger_arbiter = Arbiter::new();
    let logger_execution = async move {
        let logger_addr = Logger::new("logf.log").start();
        let _r = logger_sender.lock().unwrap().send(logger_addr);
    };
    logger_arbiter.spawn(logger_execution);
    let logger_addr = tx_l.recv().unwrap();

    logger_addr.do_send(LogMessage::new(
        "Logger inicializado"
    .to_string()));

    let argv = args().collect::<Vec<String>>();
    if argv.len() != 2 {
        logger_addr.do_send(LogMessage::new(
            "ERROR: Parametros incorrectos"
        .to_string()));    
        exit(1);
    }

    let file_path = argv[1].clone();

    let (sx, tx) = mpsc::channel();

    let addr = "localhost:8888".to_string();
    let write_stream: UdpSocket;
    let read_stream: UdpSocket;
    if let Ok(wsock) = UdpSocket::bind(addr) {
        write_stream = wsock;
        if let Ok(rsock) = write_stream.try_clone() {
            read_stream = rsock;
        } else {
            logger_addr.do_send(LogMessage::new(
                "ERROR clonando write_stream"
            .to_string()));   
            exit(1);
        }
    } else {
        logger_addr.do_send(LogMessage::new(
            "ERROR bindeando en address localhost:8888"
        .to_string()));           
        exit(1);
    }

    let mut entity_addresses = HashMap::new();

    entity_addresses.insert(EntityType::Hotel, "localhost:1234".to_string());
    entity_addresses.insert(EntityType::Bank, "localhost:1235".to_string());
    entity_addresses.insert(EntityType::Airline, "localhost:1236".to_string());

    let channel_sender = Arc::new(Mutex::new(sx));

    let sender_arbiter = Arbiter::new();
    let receiver_arbiter = Arbiter::new();

    let local_sender = channel_sender;//.clone();

    // (id, estado)
    let sender_execution = async move {
        let sender_addr = EntitySender::new(write_stream, entity_addresses).start();
        let r = local_sender.lock().unwrap().send(sender_addr);
        if r.is_err() {
            //logear error (y salir?)
        }
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
