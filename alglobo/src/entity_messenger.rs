use std::collections::HashMap;
use crate::Entity;
use actix::{Actor, Context, Handler, Message};
use std::net::UdpSocket;
use crate::transaction::{Transaction, TransactionState};

pub struct EntityMessenger {
    stream: UdpSocket,
    address_map: HashMap<Entity, String>,
    transaction_log: HashMap<u64, TransactionState>
}

impl EntityMessenger {
    pub fn new(local_addr: String, address_map: HashMap<Entity, String>) -> Self {
        let stream = UdpSocket::bind(local_addr).expect("Could not bind on that address");
        EntityMessenger { stream, address_map, transaction_log: HashMap::new() }
    }
}

impl Actor for EntityMessenger {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ServeTransaction {
    transaction: Transaction,
}

impl ServeTransaction {
    pub fn new(transaction: Transaction) -> Self {
        ServeTransaction { transaction }
    }
}

impl Handler<ServeTransaction> for EntityMessenger {
    type Result = ();

    fn handle(&mut self, msg: ServeTransaction, _ctx: &mut Self::Context) -> Self::Result {
        let v = msg.transaction.get_entities_data();
        for (entity, data) in v {
            let addr = &self.address_map[&entity];
            let data_buffer: Vec<u8> = data.into();
            println!("[MESSENGER] sending data");
            self.stream.send_to(data_buffer.as_slice(), addr).expect("falle XD");
        }
    }
}
