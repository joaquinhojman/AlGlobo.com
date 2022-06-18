use actix::{Actor, Context, Handler, Message};
use std::fmt;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct DateTime {
    /// Seconds after the minute - [0, 59]
    pub sec: i32,
    /// Minutes after the hour - [0, 59]
    pub min: i32,
    /// Hours after midnight - [0, 23]
    pub hour: i32,
    /// Day of the month - [1, 31]
    pub day: i32,
    /// Months since January - [1, 12]
    pub month: i32,
    /// Years
    pub year: i32,
}

impl DateTime {
    pub fn new() -> Self {
        Self {
            sec: 0,
            min: 0,
            hour: 0,
            day: 0,
            month: 0,
            year: 0,
        }
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.year,
            self.month,
            self.day,
            self.hour - 3,
            self.min,
            self.sec
        )
    }
}

#[derive(Clone, Debug)]
pub struct Date {
    /// Day of the month - [1, 31]
    pub day: i32,
    /// Months since January - [1, 12]
    pub month: i32,
    /// Years
    pub year: i32,
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02} UTC", self.year, self.month, self.day)
    }
}

// Convert epoch seconds into date time.
fn seconds_to_datetime(ts: i64, tm: &mut DateTime) {
    let leapyear = |year| -> bool { year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) };
    static MONTHS: [[i64; 12]; 2] = [
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
    ];
    let mut year = 1970;
    let dayclock = ts % 86400;
    let mut dayno = ts / 86400;
    tm.sec = (dayclock % 60) as i32;
    tm.min = ((dayclock % 3600) / 60) as i32;
    tm.hour = (dayclock / 3600) as i32;
    loop {
        let yearsize = if leapyear(year) { 366 } else { 365 };
        if dayno >= yearsize {
            dayno -= yearsize;
            year += 1;
        } else {
            break;
        }
    }
    tm.year = year as i32;
    let mut mon = 0;
    while dayno >= MONTHS[if leapyear(year) { 1 } else { 0 }][mon] {
        dayno -= MONTHS[if leapyear(year) { 1 } else { 0 }][mon];
        mon += 1;
    }
    tm.month = mon as i32 + 1;
    tm.day = dayno as i32 + 1;
}

pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new(file: &str) -> Self {
        let output: File;
        match OpenOptions::new().append(true).open(file) {
            Ok(file) => {
                output = file;
            }
            Err(_) => match File::create(file) {
                Ok(file) => {
                    output = file;
                }
                Err(_) => {
                    println!("voy a panickear");
                    panic!();
                }
            },
        }
        Logger { file: output }
    }

    pub fn log(&mut self, msg: String) {
        let mut tm = DateTime::new();
        if let Ok(n) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            let n_secs = n.as_secs() as i64;
            seconds_to_datetime(n_secs, &mut tm);
        }
        let _r = writeln!(self.file, "{} LOG: {:?}", tm, msg);
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

impl Actor for Logger {
    type Context = Context<Self>;
}

impl Handler<LogMessage> for Logger {
    type Result = ();

    fn handle(&mut self, log_message: LogMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.log(log_message.message);
    }
}
