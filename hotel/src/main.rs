use alglobo_common_utils::entity_logger::Logger;
use alglobo_common_utils::entity_payload::{EntityPayload, PAYLOAD_SIZE};
use alglobo_common_utils::transaction_response::TransactionResponse;
use alglobo_common_utils::transaction_state::TransactionState;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::net::UdpSocket;

use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

fn logger(rx: Receiver<String>) {
    let mut logger = Logger::new("hotel.log");
    for msg in rx {
        logger.log(msg);
    }
}

fn main() {
    let sock = UdpSocket::bind("localhost:1234").unwrap();
    let mut rng = thread_rng();
    let mut log = HashMap::new();
    let (tx, rx) = mpsc::channel();
    let _ = tx.send("Hotel inicializado en localhost:1234".to_string());

    let _ = thread::spawn(|| logger(rx));

    loop {
        let mut buf = [0; PAYLOAD_SIZE];

        let (_, addr) = sock.recv_from(&mut buf).unwrap();

        let payload_deserialized: EntityPayload = buf.to_vec().into();
        let _ = tx.send(format!("payload_deserialized: {:?}", payload_deserialized));
        let transaction_id = payload_deserialized.transaction_id;
        let response = match payload_deserialized.transaction_state {
            TransactionState::Prepare => {
                let _ = tx.send("TransactionState: Prepare".to_string());
                match log.get(&transaction_id) {
                    Some(TransactionState::Accept) | Some(TransactionState::Commit) => {
                        let _ = tx.send("TransactionResponse: Commit".to_string());
                        TransactionResponse::new(transaction_id, TransactionState::Commit)
                    }
                    Some(TransactionState::Abort) => {
                        let _ = tx.send("TransactionResponse: Abort".to_string());
                        TransactionResponse::new(transaction_id, TransactionState::Abort)
                    }
                    None => {
                        // SOLO ACA PUEDE FALLAR!!!
                        // TODO: la falla tambien puede ser un timeout esporadico, cualquiera es valida, y ambas al mismo tiempo tambien
                        let x: f64 = rng.gen();
                        if x > 0.1 {
                            log.insert(transaction_id, TransactionState::Accept);
                            let _ = tx.send("TransactionResponse: Commit".to_string());
                            TransactionResponse::new(transaction_id, TransactionState::Commit)
                        } else {
                            log.insert(transaction_id, TransactionState::Abort);
                            let _ = tx.send("FAILED. TransactionResponse: Abort".to_string());
                            TransactionResponse::new(transaction_id, TransactionState::Abort)
                        }
                    }
                    _ => panic!("Invalid transacciont state"),
                }
            }
            TransactionState::Commit => {
                let _ = tx.send("TransactionState: Commit".to_string());
                match log.get(&transaction_id) {
                    Some(TransactionState::Accept) => {
                        // TODO: escribir al log!!! si nos llego commit y todos habiamos aceptado entonces ya esta
                        // Ojito, que pasa si se falla en la fase de commit? El algoritmo este no lo maneja...
                        log.insert(transaction_id, TransactionState::Commit);
                        let _ = tx.send("TransactionResponse: Commit".to_string());
                        TransactionResponse::new(transaction_id, TransactionState::Commit)
                    }
                    Some(TransactionState::Commit) => {
                        // solo respondemos con commit, no esribimos nada porque de hecho ya lo hicimos
                        let _ = tx.send("TransactionResponse: Commit".to_string());
                        TransactionResponse::new(transaction_id, TransactionState::Commit)
                    }
                    Some(TransactionState::Abort) | None => {
                        // fallar
                        let _ = tx.send("PANICK; TransactionState::Abort cannot be handled by two fase transactionality algorithm".to_string());
                        panic!("This cannot be handled by two fase transactionality algorithm!");
                    }
                    _ => panic!("This cannot be handled by two fase transactionality algorithm!"),
                }
            }
            TransactionState::Abort => {
                let _ = tx.send("TransactionState: Abort".to_string());
                match log.get(&transaction_id) {
                    Some(TransactionState::Accept) => {
                        // liberamos los recursos, which actually means nothing here, but to write abort
                        log.insert(transaction_id, TransactionState::Abort);
                        let _ = tx.send("TransactionResponse: Abort".to_string());
                        TransactionResponse::new(transaction_id, TransactionState::Abort)
                    }
                    Some(TransactionState::Abort) => {
                        let _ = tx.send("TransactionResponse: Abort".to_string());
                        TransactionResponse::new(transaction_id, TransactionState::Abort)
                    }
                    Some(TransactionState::Commit) | None => {
                        println!("{} {:?}", transaction_id, log.get(&transaction_id));
                        let _ = tx.send("PANICK; TransactionState::Commit cannot be handled by two fase transactionality algorithm".to_string());
                        panic!("This cannot be handled by two fase transactionality algorithm!");
                    }
                    _ => panic!("This cannot be handled by two fase transactionality algorithm!"),
                }
            }
            _ => panic!("wtf"),
        };

        let response_payload: Vec<u8> = response.into();
        let _ = sock.send_to(response_payload.as_slice(), addr);
    }
}
