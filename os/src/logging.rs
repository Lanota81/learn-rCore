use log::{self, Level, LevelFilter, Log, Metadata, Record};

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn flush(&self) {}

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) { return; }
        let color = match record.level() {
            Level::Error => 31,
            Level::Warn  => 93,
            Level::Info  => 34,
            Level::Debug => 32,
            Level::Trace => 90,
        };
        println!("\x1b[{}m[{:>5}] {}\x1b[0m", 
            color, record.level(), record.args());
    }
}

pub fn init() {
    static LOGGER: SimpleLogger = SimpleLogger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG") {
        Some("ERROR") => LevelFilter::Error,
        Some("WARN")  => LevelFilter::Warn,
        Some("INFO")  => LevelFilter::Info,
        Some("DEBUG") => LevelFilter::Debug,
        Some("TRACE") => LevelFilter::Trace,
        _             => LevelFilter::Off,
    });
}