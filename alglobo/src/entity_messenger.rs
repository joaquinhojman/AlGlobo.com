use crate::entity_data::EntityData;
use crate::Entity;
use actix::{Actor, Context, Handler, Message};

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

pub struct EntityMessenger {
    whoami: Entity, // entity_socket: TcpStream,
}

impl EntityMessenger {
    pub fn new(entity: Entity) -> Self {
        EntityMessenger { whoami: entity }
    }
}

impl Actor for EntityMessenger {
    type Context = Context<Self>;
}

impl Handler<ReceiveEntityTransaction> for EntityMessenger {
    type Result = ();

    fn handle(&mut self, msg: ReceiveEntityTransaction, _ctx: &mut Self::Context) -> Self::Result {
        println!(
            "Who am i: {:?}, Transaction Entity Type: {:?}, Id Transaction: {}, Cost: {}",
            self.whoami,
            msg.entity_data.entity_type,
            msg.entity_data.transaction_id,
            msg.entity_data.cost
        );
    }
}
