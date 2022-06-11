use std::fmt;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::SystemTime;
use tracing_subscriber::Layer;

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
    file: String,
}

impl Logger {
    pub fn new(file: String) -> Self {
        Logger { file }
    }
}

impl<S> Layer<S> for Logger
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut output: File;
        match OpenOptions::new().append(true).open(self.file.clone()) {
            Ok(file) => {
                output = file;
            }
            Err(_) => match File::create(self.file.clone()) {
                Ok(file) => {
                    output = file;
                }
                Err(_) => {
                    return;
                }
            },
        }

        let mut tm = DateTime::new();
        if let Ok(n) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            let n_secs = n.as_secs() as i64;
            seconds_to_datetime(n_secs, &mut tm);
        }

        let _r = writeln!(output, "{} LOG: {:?}", tm, event.metadata().name());
        let mut visitor = PrintlnVisitor::new(output);
        event.record(&mut visitor);
    }
}

struct PrintlnVisitor {
    file: File,
}

impl PrintlnVisitor {
    pub fn new(file: File) -> Self {
        PrintlnVisitor { file }
    }
}

impl tracing::field::Visit for PrintlnVisitor {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        let _r = writeln!(self.file, "  {}={}", field.name(), value);
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        let _r = writeln!(self.file, "  {}={}", field.name(), value);
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        let _r = writeln!(self.file, "  {}={}", field.name(), value);
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        let _r = writeln!(self.file, "  {}={}", field.name(), value);
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        let _r = writeln!(self.file, "  {}={}", field.name(), value);
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        let _r = writeln!(self.file, "  {}={}", field.name(), value);
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        println!("  field={} value={:?}", field.name(), value)
    }
}
