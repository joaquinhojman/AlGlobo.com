use actix::{Actor, Context, Handler, Message};
use alglobo_common_utils::entity_logger::Logger;
use std::ops::{Deref, DerefMut};

// Wrapper utilizado para evitar duplicar codigo
// Se implementa el trait Deref para evitar hacer self.0 en todos lados
pub struct LoggerActor(Logger);

impl Deref for LoggerActor {
    type Target = Logger;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// DerefMut para poder modificar al inner
impl DerefMut for LoggerActor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl LoggerActor {
    pub fn new(file_path: &str) -> Self {
        LoggerActor(Logger::new(file_path))
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct LogMessage {
    message: String,
}

impl LogMessage {
    pub fn new(log: String) -> Self {
        LogMessage { message: log }
    }
}

impl Actor for LoggerActor {
    type Context = Context<Self>;
}

impl Handler<LogMessage> for LoggerActor {
    type Result = ();

    fn handle(&mut self, log_message: LogMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.log(log_message.message);
    }
}
