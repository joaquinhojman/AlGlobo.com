extern crate core;

pub use alglobo_common_utils;

mod beater_responder;
mod entity_receiver;
mod entity_sender;
mod file_reader;
mod logger;
mod pinger_finder;
mod statistics_handler;
mod transaction_coordinator;
mod transaction_dispatcher;
mod ok_timeout_handler;

use crate::logger::{LogMessage, LoggerActor};
use file_reader::FileReader;
use std::collections::HashMap;
use transaction_dispatcher::TransactionDispatcher;

use crate::entity_receiver::{EntityReceiver, ReceiveEntityResponse};
use crate::entity_sender::EntitySender;
use crate::file_reader::{ReadStatus, ServeNextTransaction};
use crate::statistics_handler::{LogPeriodically, StatisticsHandler};
use crate::transaction_coordinator::TransactionCoordinator;
use actix::{Actor, ActorStreamExt};
use actix_rt::{Arbiter, System};
use alglobo_common_utils::entity_type::EntityType;
use std::env::args;
use std::process::exit;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use tokio::net::UdpSocket;
use crate::beater_responder::{BeaterResponder, Responder};
use crate::ok_timeout_handler::OkTimeoutHandler;
use crate::pinger_finder::{Find, Ping, PingerFinder};

// Salen de la implementacion de Bully vista en clase
fn id_to_ctrladdr(id: usize) -> String {
    "localhost:1234".to_owned() + &*id.to_string()
}

fn id_to_dataaddr(id: usize) -> String {
    "localhost:1235".to_owned() + &*id.to_string()
}

async fn objective(file_path: String) {

}

const PROCESSES: u8 = 4;

fn main() -> Result<(), ()> {
    let actor_system = System::new();
    let argv = args().collect::<Vec<String>>();
    if argv.len() != 3 {
        // logger_addr.do_send(LogMessage::new("ERROR: Parametros incorrectos".to_string()));
        exit(1);
    }

    let pid = u8::from_str_radix(argv[2].as_str(), 10).unwrap();
    let all_pids = (0..PROCESSES).collect::<Vec<u8>>();
    let filtered_pids = all_pids.clone().into_iter().filter(|&other_pid| other_pid > pid).collect();

    actor_system.block_on(async {
        sleep(Duration::from_secs((PROCESSES - pid) as u64));

        let data_socket = Arc::new(UdpSocket::bind(id_to_dataaddr(pid as usize)).await.unwrap());
        let coordinator_socket = Arc::new(UdpSocket::bind(id_to_ctrladdr(pid as usize)).await.unwrap());

        // para sincronizar a los procesos
        /*
        let mut barrier_buffer = [0;1];
        let _ = coordinator_socket.recv_from(&mut barrier_buffer).await;

         */

        let data_clone = data_socket.clone();
        let coordinator_clone = coordinator_socket.clone();

        let timeout_handler = OkTimeoutHandler::new(pid).start();
        let timeout_handler_clone = timeout_handler.clone();

        let pinger_finder_addr = PingerFinder::new(Some(pid), pid, all_pids, filtered_pids, data_socket, coordinator_socket, timeout_handler).start();
        let pinger_clone = pinger_finder_addr.clone();
        let beater_responder_addr = BeaterResponder::new(pid, data_clone, coordinator_clone, timeout_handler_clone, pinger_finder_addr).start();
        let beater_clone = beater_responder_addr.clone();
        pinger_clone.do_send(Find::new(beater_responder_addr));
        beater_clone.do_send(Responder::new());

        actix_rt::signal::ctrl_c().await.expect("TODO: panic message");
        System::current().stop();
    });
    /*
    //Inicializacion del Actor Logger
    let (sx_l, tx_l) = mpsc::channel();
    let logger_sender = Arc::new(Mutex::new(sx_l));
    let logger_arbiter = Arbiter::new();
    let logger_execution = async move {
        let logger_addr = LoggerActor::new("logf.log").start();
        let _r = logger_sender.lock().unwrap().send(logger_addr);
    };
    logger_arbiter.spawn(logger_execution);
    let logger_addr = tx_l.recv().unwrap();

    logger_addr.do_send(LogMessage::new("Logger inicializado".to_string()));

    let argv = args().collect::<Vec<String>>();
    if argv.len() != 2 {
        logger_addr.do_send(LogMessage::new("ERROR: Parametros incorrectos".to_string()));
        exit(1);
    }

    let file_path = argv[1].clone();

    let addr = "localhost:8888".to_string();

    let mut entity_addresses = HashMap::new();
    entity_addresses.insert(EntityType::Hotel, "localhost:1234".to_string());
    entity_addresses.insert(EntityType::Bank, "localhost:1235".to_string());
    entity_addresses.insert(EntityType::Airline, "localhost:1236".to_string());
    logger_addr.do_send(LogMessage::new(format!(
        "entity_addresses: {:?}",
        entity_addresses
    )));

    actor_system.block_on(async {
        let sock = match UdpSocket::bind(&addr).await {
            Ok(sock) => sock,
            Err(what) => {
                logger_addr.do_send(LogMessage::new(format!(
                    "ERROR bindeando en {}: {}",
                    addr, what
                )));
                exit(1);
            }
        };

        let sock = Arc::new(sock);
        let statistics_handler_addr = StatisticsHandler::new().start();
        statistics_handler_addr.do_send(LogPeriodically {});
        let log_c = logger_addr.clone();
        let coordinator_addr = TransactionCoordinator::new(log_c).start();

        let log_c = logger_addr.clone();
        let write_stream = sock.clone();
        let read_stream = sock.clone();
        let coordinator_c = coordinator_addr.clone();

        let sender_addr = EntitySender::new(
            write_stream,
            entity_addresses,
            log_c,
            coordinator_c,
            statistics_handler_addr,
        )
        .start();
        let log_c = logger_addr.clone();
        let coordinator_c = coordinator_addr.clone();
        let receiver_addr = EntityReceiver::new(read_stream, log_c, coordinator_c).start();
        receiver_addr.do_send(ReceiveEntityResponse {});

        let log_c = logger_addr.clone();
        let transaction_dispatcher = TransactionDispatcher::new(sender_addr, log_c).start();

        let log_c = logger_addr.clone();
        let file_reader = match FileReader::new(file_path, transaction_dispatcher, log_c) {
            Ok(file_reader) => file_reader,
            Err(e) => {
                logger_addr.do_send(LogMessage::new(format!("ERROR: {}", e)));
                exit(1);
            }
        }
        .start();

        // esta logica no se donde debería ir
        let msg = ServeNextTransaction {};
        logger_addr.do_send(LogMessage::new("Lets read the file...".to_string()));
        while let Ok(res) = file_reader.send(msg).await {
            match res {
                ReadStatus::KeepReading => {
                    logger_addr.do_send(LogMessage::new("KeepReading".to_string()));
                }
                ReadStatus::Eof => {
                    logger_addr.do_send(LogMessage::new("EOF".to_string()));
                    break;
                }
                ReadStatus::ParseError(e) => {
                    logger_addr.do_send(LogMessage::new(format!("ERROR: {}", e)));
                    exit(1);
                }
            }
        }
        actix_rt::signal::ctrl_c()
            .await
            .expect("Could not catch signal");
        // don't sacar
        System::current().stop();
    });
    */
    match actor_system.run() {
        Ok(_) => {}
        Err(e) => {
            // logger_addr.do_send(LogMessage::new(format!("ERROR: {}", e)));
            return Err(());
        }
    }
    Ok(())
}
