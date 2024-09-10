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

use log::{LevelFilter};
use log4rs::filter::threshold::ThresholdFilter;
use log4rs::{
    append::console::ConsoleAppender,
    append::rolling_file::RollingFileAppender,
    config::{Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
};
use log4rs::append::rolling_file::policy::compound::{
    CompoundPolicy, roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger,
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

    // Define the rolling policy for all logs
    let all_roller = FixedWindowRoller::builder()
        .base(1)
        .build(&format!("{}/all_logs.{{}}.log", log_path), 5)
        .unwrap();

    let all_trigger = SizeTrigger::new(4096); // 10MB file size limit

    let all_policy = CompoundPolicy::new(Box::new(all_trigger), Box::new(all_roller));

    // Create a rolling file appender for all logs
    let all_log_file = format!("{}/all_logs.log", log_path);
    let all_file = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {l} - {m}{n}")))
        .build(all_log_file, Box::new(all_policy))
        .unwrap();

    // Define the rolling policy for filtered logs (warning and error)
    let filtered_roller = FixedWindowRoller::builder()
        .base(1)
        .build(&format!("{}/error_and_warning_logs.{{}}.log", log_path), 5)
        .unwrap();

    let filtered_trigger = SizeTrigger::new(4096); // 10MB file size limit

    let filtered_policy = CompoundPolicy::new(Box::new(filtered_trigger), Box::new(filtered_roller));

    // Create a rolling file appender for warning and error logs
    let filtered_log_file = format!("{}/error_and_warning_logs.log", log_path);
    let filtered_file = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {l} - {m}{n}")))
        .build(filtered_log_file, Box::new(filtered_policy))
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
    use log::{debug, error, info, trace, warn};

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

    #[test]
    fn test_rolling_system() {
        for _ in 0..5000 {
            error!("This is an error message");
            info!("This is an info message");
            debug!("This is a debug message");
            warn!("This is a warning message");
            trace!("This is a trace message");
        }
    }
}
