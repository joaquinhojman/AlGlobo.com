use crate::LogMessage;
use crate::Logger;
use actix::Addr;
use actix::{Actor, Context, Handler, Message};
use alglobo_common_utils::entity_type::EntityType;
use alglobo_common_utils::transaction_request::TransactionRequest;
use alglobo_common_utils::transaction_state::TransactionState;
use std::collections::HashMap;
use std::net::UdpSocket;

/*
   Actor receiver: recibir estado de transaccion (commit, abbort) + id
   -> receiver espera hasta que lleguen 3 commits para esa transaccion
   cuando llegan los 3 commits, se confirma la transaccion (se escribe en el archivo de log)
*/

pub struct EntitySender {
    stream: UdpSocket,
    address_map: HashMap<EntityType, String>,
    logger: Addr<Logger>,
    transaction_log: HashMap<u64, TransactionState>,
}

impl EntitySender {
    pub fn new(stream: UdpSocket, address_map: HashMap<EntityType, String>, logger: Addr<Logger>) -> Self {
        logger.do_send(LogMessage::new("Creating EntitySender...".to_string()));
        EntitySender {
            stream,
            address_map,
            transaction_log: HashMap::new(),
            logger
        }
    }
}

impl Actor for EntitySender {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ServeTransaction {
    transaction: TransactionRequest,
}

impl ServeTransaction {
    pub fn new(transaction: TransactionRequest) -> Self {
        ServeTransaction { transaction }
    }
}

impl Handler<ServeTransaction> for EntitySender {
    type Result = ();

    fn handle(&mut self, msg: ServeTransaction, _ctx: &mut Self::Context) -> Self::Result {
        let v = msg.transaction.get_entities_data();
        self.logger.do_send(LogMessage::new(format!("entities_data: {:?}", v)));
        for (entity, data) in v {
            let addr = &self.address_map[&entity];
            let data_buffer: Vec<u8> = data.into();
            println!("[MESSENGER] sending data");
            self.logger.do_send(LogMessage::new(format!("[MESSENGER] sending data: {:?}", data_buffer.clone())));
            let _r = self.stream.send_to(data_buffer.as_slice(), addr);
            //en caso de que falle el socket tendriamos que agregarlo a
            //las transacciones fallidas y hacerla otra vez?
        }
    }
}
