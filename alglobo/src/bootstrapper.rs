use crate::entity_sender::RegisterFileReader;
use crate::file_writer::FileWriter;
use crate::{
    EntityReceiver, EntitySender, FileReader, LogMessage, LoggerActor, ReadStatus,
    ReceiveEntityResponse, ServeNextTransaction, StatisticsHandler, TransactionCoordinator,
    TransactionDispatcher,
};
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, WrapFuture};
use alglobo_common_utils::entity_type::EntityType;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::UdpSocket;

pub struct Bootstrapper {
    file_path: String,
}

impl Actor for Bootstrapper {
    type Context = Context<Self>;
}

impl Bootstrapper {
    pub fn new(file_path: String) -> Self {
        Bootstrapper { file_path }
    }

    async fn run(logger_addr: Addr<LoggerActor>, file_path: String) {
        let addr = "localhost:8888".to_string();
        let mut entity_addresses = HashMap::new();
        entity_addresses.insert(EntityType::Hotel, "localhost:1234".to_string());
        entity_addresses.insert(EntityType::Bank, "localhost:1235".to_string());
        entity_addresses.insert(EntityType::Airline, "localhost:1236".to_string());

        let sock = match UdpSocket::bind(&addr).await {
            Ok(sock) => sock,
            Err(what) => {
                logger_addr.do_send(LogMessage::new(format!(
                    "ERROR bindeando en {}: {}",
                    addr, what
                )));
                panic!("ERROR bindeando en {}: {}", addr, what);
            }
        };

        let sock = Arc::new(sock);

        let statistics_handler_addr = StatisticsHandler::new().start();

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
        let sender_clone = sender_addr.clone();
        let transaction_dispatcher = TransactionDispatcher::new(sender_addr, log_c).start();

        let log_c = logger_addr.clone();
        let file_writer = match FileWriter::new("failed_transactions.csv".to_string(), log_c) {
            Ok(file_writer) => file_writer,
            Err(e) => {
                logger_addr.do_send(LogMessage::new(format!("ERROR: {}", e)));
                panic!("ERROR: {}", e)
            }
        }
        .start();

        let log_c = logger_addr.clone();
        let file_reader =
            match FileReader::new(file_path, transaction_dispatcher, file_writer, log_c) {
                Ok(file_reader) => file_reader,
                Err(e) => {
                    logger_addr.do_send(LogMessage::new(format!("ERROR: {}", e)));
                    panic!("ERROR: {}", e)
                }
            }
            .start();
        sender_clone.do_send(RegisterFileReader::new(file_reader.clone()));

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
                    panic!("ERROR: {}", e)
                }
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RunAlGlobo {
    logger_addr: Addr<LoggerActor>,
}

impl RunAlGlobo {
    pub fn new(logger_addr: Addr<LoggerActor>) -> Self {
        RunAlGlobo { logger_addr }
    }
}

impl Handler<RunAlGlobo> for Bootstrapper {
    type Result = ();

    fn handle(&mut self, msg: RunAlGlobo, ctx: &mut Self::Context) -> Self::Result {
        println!("[BOOTSTRAPPER] spawning alglobo schedule");
        let path = self.file_path.clone();
        ctx.spawn(Bootstrapper::run(msg.logger_addr, path).into_actor(self));
    }
}
