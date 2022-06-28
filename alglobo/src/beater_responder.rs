use crate::beater_responder::ResponderState::{Continue, FindNew, StartPing};
use crate::ok_timeout_handler::{OkTimeoutHandler, RegisterOkReceived};
use crate::pinger_finder::{Find, PingerFinder, SetNewLeader};
use crate::{id_to_ctrladdr, Ping};
use actix::{
    Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, ResponseActFuture,
    WrapFuture,
};
use futures::future::join_all;
use std::sync::Arc;
use tokio::net::UdpSocket;

// otra fsm
// saltamos del estado de BEAT (cuando soy el lider) al estado de RESPONDER (cuando no lo soy)
// y viceversa
pub struct BeaterResponder {
    pid: u8,
    data_socket: Arc<UdpSocket>,
    coordinator_socket: Arc<UdpSocket>,
    ok_timeout_handler_addr: Addr<OkTimeoutHandler>,
    pinger_finder_addr: Addr<PingerFinder>,
}

impl BeaterResponder {
    pub fn new(
        pid: u8,
        data_socket: Arc<UdpSocket>,
        coordinator_socket: Arc<UdpSocket>,
        ok_timeout_handler_addr: Addr<OkTimeoutHandler>,
        pinger_finder_addr: Addr<PingerFinder>,
    ) -> Self {
        BeaterResponder {
            pid,
            data_socket,
            coordinator_socket,
            ok_timeout_handler_addr,
            pinger_finder_addr,
        }
    }
}

impl Actor for BeaterResponder {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Beat {}

impl Handler<Beat> for BeaterResponder {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: Beat, _: &mut Self::Context) -> Self::Result {
        let sock = self.data_socket.clone();
        let my_pid = self.pid;
        let mut buf = [0; 4];
        let fut = async move {
            if let Ok((_, addr)) = sock.recv_from(&mut buf).await {
                // logger("recibi <buf (PING)> de <addr>");
                // let it crash
                println!(
                    "[BEATER from PID {}] received {}",
                    my_pid,
                    String::from_utf8_lossy(buf.as_slice())
                );
                sock.send_to("PONG".as_bytes(), addr).await.unwrap();
            }
        };

        Box::pin(fut.into_actor(self).map(|_, _, ctx| {
            ctx.address().do_send(msg);
        }))
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Responder {}

impl Responder {
    pub fn new() -> Self {
        Responder {}
    }
}

enum ResponderState {
    FindNew,
    Continue,
    StartPing(u8),
}

impl Handler<Responder> for BeaterResponder {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: Responder, _ctx: &mut Self::Context) -> Self::Result {
        let sock = self.coordinator_socket.clone();
        let my_pid = self.pid;
        let addr_timeout_handler = self.ok_timeout_handler_addr.clone();

        let fut = async move {
            let mut buf = [0; 2];
            if let Ok((_, addr)) = sock.recv_from(&mut buf).await {
                let id_from = buf[1];
                match &buf[0] {
                    b'O' => {
                        println!("[PID {}] received OK from {}", my_pid, id_from);
                        addr_timeout_handler.do_send(RegisterOkReceived {});
                        Continue
                    }
                    b'E' => {
                        println!("[PID {}] received ELECTION from {}", my_pid, id_from);
                        // enviar ok si era mas chico que yo
                        // mensaje de find_new()
                        if id_from < my_pid {
                            let mut res = [0; 2];
                            res[0] = b'O';
                            res[1] = my_pid;
                            sock.send_to(res.as_slice(), addr).await.unwrap();
                            return FindNew;
                        }
                        Continue
                    }
                    b'C' => {
                        println!("[PID {}] received COORDINATOR from {}", my_pid, id_from);
                        // notifico por queue que hay un lider nuevo
                        StartPing(id_from)
                    }
                    _ => {
                        println!("WTF");
                        Continue
                    }
                }
            } else {
                Continue
            }
        };
        Box::pin(fut.into_actor(self).map(|state, me, ctx| {
            match state {
                FindNew => {
                    me.pinger_finder_addr.do_send(Find::new(ctx.address()));
                }
                StartPing(ping_id) => {
                    me.pinger_finder_addr.do_send(SetNewLeader::new(ping_id));
                    me.pinger_finder_addr
                        .do_send(Ping::new(ping_id, ctx.address()));
                }
                _ => {}
            }
            ctx.address().do_send(msg)
        }))
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastCoordinator {
    all_pids: Vec<u8>,
}

impl BroadcastCoordinator {
    pub fn new(all_pids: Vec<u8>) -> Self {
        BroadcastCoordinator { all_pids }
    }
}

impl Handler<BroadcastCoordinator> for BeaterResponder {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: BroadcastCoordinator, _ctx: &mut Self::Context) -> Self::Result {
        let socket = self.coordinator_socket.clone();
        let my_pid = self.pid;

        let fut = async move {
            let buffer_coordinator = vec![b'C', my_pid];
            let mut futures_buffer = vec![];
            for other_pid in msg.all_pids {
                if other_pid != my_pid {
                    futures_buffer.push(socket.send_to(
                        buffer_coordinator.as_slice(),
                        id_to_ctrladdr(other_pid as usize),
                    ));
                }
            }
            join_all(futures_buffer).await;
        };

        Box::pin(fut.into_actor(self).map(|_, _, ctx| {
            ctx.address().do_send(Beat {});
        }))
    }
}
