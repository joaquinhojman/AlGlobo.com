use actix::{Actor, Addr, Context, Handler, Message};

pub struct TransactionDispatcher {
    entities_mailboxes: Vec<()>,
}

impl TransactionDispatcher {
    pub fn new() -> Self {
        TransactionDispatcher {
            entities_mailboxes: Vec::new(),
        }
    }
}

impl Actor for TransactionDispatcher {
    type Context = Context<Self>;
}
