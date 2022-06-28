use crate::beater_responder::{Beat, BeaterResponder};
use crate::ok_timeout_handler::{OkTimeoutHandler, WaitTimeout};
use crate::{id_to_ctrladdr, id_to_dataaddr, Responder};
use actix::{
    Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, ResponseActFuture,
    WrapFuture,
};
use futures::future::join_all;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::oneshot::{Receiver, Sender};
use tokio::time::{sleep, timeout};

const PING: &[u8] = "PING".as_bytes();
const PING_PONG_SIZE: usize = 4;
// generous amount of timeout
const TIMEOUT_S: u64 = 10;
const PING_RATE_S: u64 = 2;

// FSM entre Ping y Find
// en ningun estado soy lider, pero puedo pasar a serlo luego de Find
pub struct PingerFinder {
    leader: Option<u8>,
    pid: u8,
    all_pids: Vec<u8>,
    filtered_pids: Vec<u8>,
    data_socket: Arc<UdpSocket>,
    coordinator_socket: Arc<UdpSocket>,
    ok_timeout_handler_addr: Addr<OkTimeoutHandler>,
}

impl PingerFinder {
    pub fn new(
        leader: Option<u8>,
        pid: u8,
        all_pids: Vec<u8>,
        filtered_pids: Vec<u8>,
        data_socket: Arc<UdpSocket>,
        coordinator_socket: Arc<UdpSocket>,
        ok_timeout_handler_addr: Addr<OkTimeoutHandler>,
    ) -> Self {
        PingerFinder {
            leader,
            pid,
            all_pids,
            filtered_pids,
            data_socket,
            coordinator_socket,
            ok_timeout_handler_addr,
        }
    }
}

impl Actor for PingerFinder {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Ping {
    ping_id: u8,
    responder: Addr<BeaterResponder>,
}

impl Ping {
    pub fn new(ping_id: u8, responder: Addr<BeaterResponder>) -> Self {
        Ping { ping_id, responder }
    }
}

impl Handler<Ping> for PingerFinder {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: Ping, _: &mut Self::Context) -> Self::Result {
        let sock = self.data_socket.clone();
        let my_pid = self.pid;
        let fut = async move {
            sock.send_to(PING, id_to_dataaddr(msg.ping_id as usize))
                .await
                .expect("rip");
            let mut buf = [0; PING_PONG_SIZE];
            let recv_fut = sock.recv_from(&mut buf);
            match timeout(Duration::from_secs(TIMEOUT_S), recv_fut).await {
                Ok(_) => {
                    // avoid ping ddos
                    println!(
                        "[PID {}] received {:?}",
                        my_pid,
                        String::from_utf8_lossy(buf.as_slice())
                    );
                    sleep(Duration::from_secs(PING_RATE_S)).await;
                    Ok(())
                }
                Err(_) => Err(()),
            }
        };
        Box::pin(fut.into_actor(self).map(|res, _, ctx| match res {
            Ok(_) => {
                // si es ok sigo pingeando
                ctx.address().do_send(msg)
            }
            // si es error busco un nuevo lider
            Err(_) => {
                println!("Error");
                ctx.address().do_send(Find::new(msg.responder))
            }
        }))
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Find {
    responder: Addr<BeaterResponder>,
}

impl Find {
    pub fn new(responder: Addr<BeaterResponder>) -> Self {
        Find { responder }
    }
}

impl Handler<Find> for PingerFinder {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: Find, ctx: &mut Self::Context) -> Self::Result {
        // 1: enviar mensaje election a todos los que tengan mayor id
        // 2: esperar hasta timeout que me respondan
        // 2a: si me responden, me quedo calmadito y espero a que alguien se declare lider
        // 2b: si no me responden, soy el lider, y le aviso al resto

        // flag: si esto esta seteado en None, ya pasamos por acá
        // entonces volvemos sin mas
        // de hecho, puede que el mismo mensaje esté esperando el timeout de Ok
        if self.leader.is_none() {
            return Box::pin(std::future::ready(()).into_actor(self));
        }

        // seteamos el leader en None, ya que si estamos acá es porque fallo ping (y/o no hay lider)
        self.leader = None;

        let timeout_handler_addr = self.ok_timeout_handler_addr.clone();
        let sock = self.coordinator_socket.clone();
        let my_pid = self.pid;
        let filtered_pids = self.filtered_pids.clone();
        let send_buffer = vec![b'E', my_pid];
        let responder = msg.responder.clone();
        let all_pids = self.all_pids.clone();

        let fut = async move {
            let mut send_futures = vec![];
            for pid in &filtered_pids {
                send_futures
                    .push(sock.send_to(send_buffer.as_slice(), id_to_ctrladdr(*pid as usize)));
            }
            // mandar al ok timeout handler que empiece a escuchar
            // esto esta antes de tal manera de evitar race conditions
            timeout_handler_addr.do_send(WaitTimeout::new(responder, all_pids));
            // envio todos los mensajes
            let v_res = join_all(send_futures).await;
            // como se handlea este caso??
            if !v_res.into_iter().any(|res| res.is_ok()) {
                // leader
                // panic!()
            }
        };
        Box::pin(fut.into_actor(self).map(|_, _, _| {}))
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SetNewLeader {
    leader: u8,
}

impl SetNewLeader {
    pub fn new(leader: u8) -> Self {
        SetNewLeader { leader }
    }
}

impl Handler<SetNewLeader> for PingerFinder {
    type Result = ();

    fn handle(&mut self, msg: SetNewLeader, ctx: &mut Self::Context) -> Self::Result {
        self.leader = Some(msg.leader);
    }
}
