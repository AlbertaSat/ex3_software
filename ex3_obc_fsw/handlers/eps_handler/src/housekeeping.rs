/*
Written by Devin Headrick
Summer 2024

*/
use std::fmt;

use crate::enums::OutputChannelStates;

/// This is a struct that contains HK DATA THAT WE REGULARLY CARE ABOUT AND MAY WANT TO PUT IN BEACON
/// THIS DOES NOT INCLUDE 'ALL' HK DATA , THE EPS HAS A LOT IT SPITS OUT WHEN 'INSTANTANEOUS TELEMETRY' IS REQUESTED - we only care about some regularly
pub struct Housekeeping {
    boot_count: u16,
    uptime: u32,
    battery_voltage: f32,
    battery_current_in: f32,
    battery_current_out: f32,
    mppt_mode: u8,
    gs_watchdog_time_left: u32,
    current_time_unix: u32,
    output_channel_states: [OutputChannelStates; 18],
}

impl Housekeeping {
    pub fn new_default() -> Self {
        Housekeeping {
            boot_count: 0,
            uptime: 0,
            battery_voltage: 0.0,
            battery_current_in: 0.0,
            battery_current_out: 0.0,
            mppt_mode: 0,
            gs_watchdog_time_left: 0,
            current_time_unix: 0,
            output_channel_states: [OutputChannelStates::Off; 18],
        }
    }
}

// impl fmt::Display for Housekeeping {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "Boot Count: {}\nUptime: {}\nBattery Voltage: {}\nBattery Current In: {}\nBattery Current Out: {}\nGS Watchdog Timer: {}\nCurrent Time Unix: {}\nOutput Channel States: {:?}",
//             self.boot_count,
//             self.uptime,
//             self.battery_voltage,
//             self.battery_current_in,
//             self.battery_current_out,
//             self.gs_watchdog_time_left,
//             self.current_time_unix,
//             self.output_channel_states
//         )
//     }
// }
