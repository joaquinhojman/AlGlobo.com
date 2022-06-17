use crate::entity_data::EntityData;
use crate::Entity;
use actix::{Actor, Context, Handler, Message};
use std::io::Write;
use std::net::TcpStream;

pub struct EntityMessenger {
    _entity: Entity,
    stream: TcpStream,
}

impl EntityMessenger {
    pub fn new(_entity: Entity, addr: String) -> Self {
        let stream = TcpStream::connect(addr).unwrap();
        EntityMessenger { _entity, stream }
    }
}

impl Actor for EntityMessenger {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ReceiveEntityTransaction {
    entity_data: EntityData,
}

impl ReceiveEntityTransaction {
    pub fn new(entity_data: EntityData) -> Self {
        ReceiveEntityTransaction { entity_data }
    }
}

impl Handler<ReceiveEntityTransaction> for EntityMessenger {
    type Result = ();

    fn handle(&mut self, msg: ReceiveEntityTransaction, _ctx: &mut Self::Context) -> Self::Result {
        println!(
            "{}, {}",
            msg.entity_data.transaction_id, msg.entity_data.cost
        );
        let to_send: Vec<u8> = msg.entity_data.into();
        match self.stream.write_all(to_send.as_slice()) {
            Ok(()) => println!("written"),
            Err(e) => println!("{}", e),
        }
    }
}
