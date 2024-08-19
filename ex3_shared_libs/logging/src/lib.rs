/*
Written by Rowan Rasmusson Devin Headrick
Summer 2024

So how should this work?? - other processes call the init logger fxn to init a logger for their process
    - they pass this a path to specify files logs are written to
    - That's what this does right now.

TODOs:
    - Programmatically allow the console log level to be set (e.g. for debugging v.s. demos)
    - Setup functions to route logs to different files based on their associated FSW component
    - Setting rolling file size limits for log files
    - Setup functions to filter logs based on
        - Severity
        - Component
        - Time

*/

use log::{debug, error, info, trace, warn};
use log::{Level, LevelFilter};
use log4rs::filter::threshold::ThresholdFilter;
use log4rs::{
    append::console::ConsoleAppender,
    append::file::FileAppender,
    config::{Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
};

fn configure_logger(
    all_log_level: LevelFilter,
    filtered_log_level: LevelFilter,
    log_path: &str,
) -> Config {
    // Create a console appender
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{h({l})} {m}{n}")))
        .build();

    // Create a file appender for all logs
    let all_log_file = format!("{}/all_logs.log", log_path);
    let all_file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {l} - {m}{n}")))
        .build(all_log_file)
        .unwrap();

    // Create a file appender for warning and error logs
    let filtered_log_file = format!("{}/error_and_warning_logs.log", log_path);
    let filtered_file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {l} - {m}{n}")))
        .build(filtered_log_file)
        .unwrap();

    let filtered_file = Appender::builder()
        .filter(Box::new(ThresholdFilter::new(filtered_log_level)))
        .build("filtered_file", Box::new(filtered_file));

    // Build the configuration
    Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("all_file", Box::new(all_file)))
        .appender(filtered_file)
        .logger(
            Logger::builder()
                .appender("all_file")
                .additive(false)
                .build("all_logs", all_log_level),
        )
        .logger(
            Logger::builder()
                .appender("filtered_file")
                .additive(false)
                .build("filtered_logs", filtered_log_level),
        )
        .build(
            Root::builder()
                .appender("stdout")
                .appender("all_file")
                .appender("filtered_file")
                .build(all_log_level),
        )
        .unwrap()
}

pub fn init_logger(log_path: &str) {
    let all_log_levels = LevelFilter::Trace;
    let warnings_and_error_log_levels = LevelFilter::Warn;

    let config = configure_logger(all_log_levels, warnings_and_error_log_levels, log_path);

    // Initialize the logger
    let _handle = log4rs::init_config(config).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_severities() {
        let log_path = "logs"; // Specify your log directory
        init_logger(log_path);
        error!("This is an error message");
        info!("This is an info message");
        debug!("This is a debug message");
        warn!("This is a warning message");
        trace!("This is a trace message");
    }
}
