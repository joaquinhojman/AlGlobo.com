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
mod bootstrapper;

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

use std::env::args;
use std::process::exit;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use tokio::net::UdpSocket;
use crate::beater_responder::{BeaterResponder, Responder};
use crate::bootstrapper::Bootstrapper;
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

        let bootstrapper = Bootstrapper::new("transactions.csv".to_string()).start();
        let timeout_handler = OkTimeoutHandler::new(pid, bootstrapper, logger_addr).start();
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

    match actor_system.run() {
        Ok(_) => {}
        Err(e) => {
            // logger_addr.do_send(LogMessage::new(format!("ERROR: {}", e)));
            return Err(());
        }
    }
    Ok(())
}
