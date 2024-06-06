use log::{debug, error, info};
use log4rs;

pub fn init_logger() {
    debug!("Initializing Logger.");
    if let Err(e) = log4rs::init_file("scheduler/log4rs.toml", Default::default()) {
        println!("Error initializing log4rs: {}", e);
    } else {
        println!("Logger initialized successfully.");
    }
}

pub fn log_error(e: String, id: u32) {
    error!("{} ID: {}", e, id);
}

pub fn log_info(e: String, id: u32) {
    info!("{} ID: {}", e, id);
}