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

impl Default for DateTime {
    fn default() -> Self {
        Self::new()
    }
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

#[cfg(test)]
mod tests {
    use crate::entity_logger::File;
    use crate::entity_logger::Logger;
    use std::io;
    use std::io::BufRead;
    use std::time::SystemTime;

    use super::seconds_to_datetime;
    use super::DateTime;

    #[test]
    fn test_logger() {
        let filename = "test.log";
        let mut logger = Logger::new(filename);
        logger.log("TEST".to_string());

        let file = File::open(filename).unwrap();
        let lines = io::BufReader::new(file).lines();
        for line in lines {
            let l = line.unwrap();
            let slice = &l[26..30];
            assert_eq!(slice, "TEST");
            break;
        }
    }

    #[test]
    fn test_datetime() {
        let mut tm = DateTime::new();
        if let Ok(n) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            let n_secs = n.as_secs() as i64;
            seconds_to_datetime(n_secs, &mut tm);
        }
        let datetime = format!("{}", tm);
        assert_eq!("2022", &datetime[0..4]);
        assert_eq!("06", &datetime[5..7]);
    }
}
