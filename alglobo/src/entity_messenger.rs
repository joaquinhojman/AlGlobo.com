use actix::{Actor, Context, Handler, Message};
use tracing::info;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ReceiveEntityTransaction {
    tid: i64,
    cost: i64,
}

impl ReceiveEntityTransaction {
    pub fn new(tid: i64, cost: i64) -> Self {
        ReceiveEntityTransaction { tid, cost }
    }
}

pub struct EntityMessenger {
    id: u8, // entity_socket: TcpStream,
}

impl EntityMessenger {
    pub fn new(id: u8) -> Self {
        EntityMessenger { id }
    }
}

impl Actor for EntityMessenger {
    type Context = Context<Self>;
}

impl Handler<ReceiveEntityTransaction> for EntityMessenger {
    type Result = ();

    fn handle(&mut self, msg: ReceiveEntityTransaction, _ctx: &mut Self::Context) -> Self::Result {
        println!("id:{:?}, tid: {:?}, cost: {:?}", self.id, msg.tid, msg.cost);
        info!(
            event = "ReceiveEntityTransaction",
            id = &(self.id).to_string()[..],
            tid = &(msg.tid).to_string()[..],
            cost = &(msg.cost).to_string()[..]
        );
    }
}
