use std::collections::HashSet;
use std::thread::sleep;
use std::time::Duration;

use actix::{Actor, ActorFutureExt, AsyncContext, Context, ContextFutureSpawner, Handler, Message, ResponseActFuture, WrapFuture};


const LOG_PERIOD_S: u64 = 1;

// TODO: cambiar este nombre de mierda
pub struct StatisticsHandler {
    transaction_id_timestamp_set: HashSet<u64>,
    total_transactions: u64,
    current_finished_transactions: u64,
    elapsed_time: Duration,
}

impl StatisticsHandler {
    pub fn new() -> Self {
        StatisticsHandler {
            transaction_id_timestamp_set: HashSet::new(),
            total_transactions: 0,
            current_finished_transactions: 0,
            elapsed_time: Duration::from_secs(0),
        }
    }
}

impl Actor for StatisticsHandler {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterTransaction {
    transaction_id: u64,
}

impl RegisterTransaction {
    pub fn new(transaction_id: u64) -> Self {
        RegisterTransaction { transaction_id }
    }
}

impl Handler<RegisterTransaction> for StatisticsHandler {
    type Result = ();

    fn handle(&mut self, msg: RegisterTransaction, _ctx: &mut Self::Context) -> Self::Result {
        // si ya estaba y la estamos tratando de registrar de nuevo es un bug
        if self
            .transaction_id_timestamp_set
            .get(&msg.transaction_id)
            .is_some()
        {
            return;
        }

        self.transaction_id_timestamp_set.insert(msg.transaction_id);
        self.total_transactions += 1;
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UnregisterTransaction {
    transaction_id: u64,
    duration: Duration,
}

impl UnregisterTransaction {
    pub fn new(transaction_id: u64, duration: Duration) -> Self {
        UnregisterTransaction {
            transaction_id,
            duration,
        }
    }
}

impl Handler<UnregisterTransaction> for StatisticsHandler {
    type Result = ();

    fn handle(&mut self, msg: UnregisterTransaction, _ctx: &mut Self::Context) -> Self::Result {
        // si tratamos de desregistrar una transaccion y no existe es un bug
        if self
            .transaction_id_timestamp_set
            .get(&msg.transaction_id)
            .is_none()
        {
            return;
        }

        let _ = self
            .transaction_id_timestamp_set
            .remove(&msg.transaction_id);
        self.current_finished_transactions += 1;
        self.elapsed_time += msg.duration;
    }
}

#[derive(Message)]
#[rtype(result = "f64")]
pub struct GetMeanDuration {}

impl Handler<GetMeanDuration> for StatisticsHandler {
    type Result = ResponseActFuture<Self, f64>;

    fn handle(&mut self, _msg: GetMeanDuration, _ctx: &mut Self::Context) -> Self::Result {
        Box::pin({
            std::future::ready(()).into_actor(self).map(|_, me, ctx| {
                if me.total_transactions == 0 {
                    return 0.0;
                } else {
                    me.elapsed_time.as_secs() as f64 / me.total_transactions as f64
                }
            })
        })
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct LogPeriodically {}

// TODO: refactor
impl Handler<LogPeriodically> for StatisticsHandler {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: LogPeriodically, ctx: &mut Self::Context) -> Self::Result {
        Box::pin(
            actix_rt::time::sleep(Duration::from_secs(LOG_PERIOD_S))
                .into_actor(self)
                .map(|_, me, ctx| {
                    let mean_time = if me.current_finished_transactions == 0 {
                        0.0
                    } else {
                        me.elapsed_time.as_secs() as f64 / me.current_finished_transactions as f64
                    };
                    let tps = me.current_finished_transactions as f64 / mean_time;
                    println!("[STATS]\n\t- Total transactions: {}\n\t- Finished Transactions: {}\n\t- Mean time {}s\n\t- Finished transactions per second: {}\n", me.total_transactions, me.current_finished_transactions, mean_time, tps);
                    ctx.address().do_send(LogPeriodically {})
                })
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::statistics_handler::{
        GetMeanDuration, RegisterTransaction, StatisticsHandler, UnregisterTransaction,
    };
    use actix::Actor;
    use float_cmp::approx_eq;
    use std::time::Duration;

    #[actix_rt::test]
    async fn test_no_transactions_shields_0_seconds() {
        let addr = StatisticsHandler::new().start();

        let secs = addr
            .send(GetMeanDuration {})
            .await
            .expect("fallo envio de mensaje de log");

        assert!(approx_eq!(f64, secs, 0.0, epsilon = 1e-9));
    }

    #[actix_rt::test]
    async fn test_one_transaction_takes_3_seconds_and_mean_time_is_3_seconds() {
        let addr = StatisticsHandler::new().start();
        addr.send(RegisterTransaction::new(0))
            .await
            .expect("fallo el envio del registro de transaccion");

        let d_3_seconds = Duration::from_secs(3);

        addr.send(UnregisterTransaction::new(0, d_3_seconds))
            .await
            .expect("fallo el envio de desregistrar la transaccion");

        let secs = addr
            .send(GetMeanDuration {})
            .await
            .expect("fallo envio de mensaje de log");

        assert!(approx_eq!(f64, secs, 3.0, epsilon = 1e-9));
    }

    #[actix_rt::test]
    async fn test_two_transactions_take_5_seconds_in_total_and_mean_time_is_2dot5_seconds() {
        let addr = StatisticsHandler::new().start();

        addr.send(RegisterTransaction::new(0))
            .await
            .expect("fallo el envio del registro de transaccion");
        addr.send(RegisterTransaction::new(1))
            .await
            .expect("fallo el envio del registro de transaccion");

        let d_3_seconds = Duration::from_secs(3);
        let d_2_seconds = Duration::from_secs(2);

        addr.send(UnregisterTransaction::new(0, d_3_seconds))
            .await
            .expect("fallo el envio de desregistrar la transaccion");
        addr.send(UnregisterTransaction::new(1, d_2_seconds))
            .await
            .expect("fallo el envio de desregistrar la transaccion");

        let secs = addr
            .send(GetMeanDuration {})
            .await
            .expect("fallo envio de mensaje de log");

        assert!(approx_eq!(f64, secs, 2.5, epsilon = 1e-9));
    }

    #[actix_rt::test]
    async fn test_four_transactions_take_32_seconds_in_total_and_mean_time_is_8_seconds() {
        let addr = StatisticsHandler::new().start();

        addr.send(RegisterTransaction::new(0))
            .await
            .expect("fallo el envio del registro de transaccion");
        addr.send(RegisterTransaction::new(1))
            .await
            .expect("fallo el envio del registro de transaccion");
        addr.send(RegisterTransaction::new(2))
            .await
            .expect("fallo el envio del registro de transaccion");
        addr.send(RegisterTransaction::new(3))
            .await
            .expect("fallo el envio del registro de transaccion");

        let d_8_seconds = Duration::from_secs(8);

        addr.send(UnregisterTransaction::new(0, d_8_seconds))
            .await
            .expect("fallo el envio de desregistrar la transaccion");
        addr.send(UnregisterTransaction::new(1, d_8_seconds))
            .await
            .expect("fallo el envio de desregistrar la transaccion");
        addr.send(UnregisterTransaction::new(2, d_8_seconds))
            .await
            .expect("fallo el envio de desregistrar la transaccion");
        addr.send(UnregisterTransaction::new(3, d_8_seconds))
            .await
            .expect("fallo el envio de desregistrar la transaccion");

        let secs = addr
            .send(GetMeanDuration {})
            .await
            .expect("fallo envio de mensaje de log");

        assert!(approx_eq!(f64, secs, 8.0, epsilon = 1e-9));
    }
}
