use std::io::Write;

use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;

/// logger config - level filter : Debug and print timestamp
pub fn log_config() {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Error)
        .parse_env("SOURCERER_LOG")
        .init();
}
