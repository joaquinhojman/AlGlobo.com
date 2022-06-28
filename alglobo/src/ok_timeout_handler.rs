use crate::beater_responder::BroadcastCoordinator;
use crate::bootstrapper::{Bootstrapper, RunAlGlobo};
use crate::{BeaterResponder, LoggerActor};
use actix::{
    Actor, ActorFutureExt, Addr, Context, Handler, Message, ResponseActFuture, WrapFuture,
};
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::timeout;

pub struct OkTimeoutHandler {
    sender: Option<oneshot::Sender<u8>>,
    receiver: Option<oneshot::Receiver<u8>>,
    pid: u8,
    bootstrapper: Addr<Bootstrapper>,
    logger: Addr<LoggerActor>,
}

impl OkTimeoutHandler {
    pub fn new(pid: u8, bootstrapper: Addr<Bootstrapper>, logger: Addr<LoggerActor>) -> Self {
        let (tx, rx) = oneshot::channel();
        OkTimeoutHandler {
            sender: Some(tx),
            receiver: Some(rx),
            pid,
            bootstrapper,
            logger,
        }
    }
}

impl Actor for OkTimeoutHandler {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct WaitTimeout {
    responder: Addr<BeaterResponder>,
    all_pids: Vec<u8>,
}

impl WaitTimeout {
    pub fn new(responder: Addr<BeaterResponder>, all_pids: Vec<u8>) -> Self {
        WaitTimeout {
            responder,
            all_pids,
        }
    }
}

impl Handler<WaitTimeout> for OkTimeoutHandler {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: WaitTimeout, _: &mut Self::Context) -> Self::Result {
        if self.receiver.is_none() || self.sender.is_none() {
            let (tx, rx) = oneshot::channel();
            self.sender = Some(tx);
            self.receiver = Some(rx);
        }

        let rx = self.receiver.take().unwrap();
        let pid = self.pid;
        let bootstrapper = self.bootstrapper.clone();
        let logger = self.logger.clone();

        let fut = async move {
            match timeout(Duration::from_secs(10), rx).await {
                Ok(_) => {
                    println!("[PID {}] ok, no soy coordinator, espero", pid);
                    // todavia no sabes cual es el coordinador
                    // esperar coordinador
                    Ok(())
                }
                Err(_) => {
                    println!("[PID {}] timeout, mandando coordinator", pid);
                    msg.responder
                        .do_send(BroadcastCoordinator::new(msg.all_pids));
                    bootstrapper.do_send(RunAlGlobo::new(logger));
                    Err(())
                }
            }
        };
        Box::pin(fut.into_actor(self).map(|_, _, _ctx| {}))
    }
}

// lo llama el responder cuando llega un ok
#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterOkReceived {}

impl Handler<RegisterOkReceived> for OkTimeoutHandler {
    type Result = ();

    fn handle(&mut self, _msg: RegisterOkReceived, _ctx: &mut Self::Context) -> Self::Result {
        if self.sender.is_some() {
            let tx = self.sender.take().unwrap();
            // envio fruta, total es para despertar al timeout
            tx.send(0).unwrap();
        }
    }
}
