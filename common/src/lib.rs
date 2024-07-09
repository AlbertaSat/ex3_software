/// Ports used for communication between handlers and simulated subsystems / payloads
pub mod ports {
    pub const SIM_DFGM_PORT: u16 = 1802;
    pub const SIM_ADCS_PORT: u16 = 1803;
    pub const SIM_EPS_PORT: u16 = 1804;
    pub const SIM_COMMS_PORT: u16 = 1805;
    pub const SIM_IRIS_PORT: u16 = 1806;

    pub const DFGM_HANDLER_DISPATCHER_PORT: u16 = 1900;
    pub const SCHEDULER_DISPATCHER_PORT: u16 = 1901;
    pub const SUBSYSTEM_MONITOR_DISPATCHER_PORT: u16 = 1902;
    pub const BULK_MSG_HANDLER_DISPATCHER_PORT: u16 = 1903;
}

/// Each thing that can emit or receive a message has an associated ID. Each message header includes this id for source and destination.
/// Referencing this page:
pub mod component_ids {
    pub const OBC: u8 = 0;
    pub const EPS: u8 = 1;
    pub const ADCS: u8 = 2;
    pub const DFGM: u8 = 3;
    pub const IRIS: u8 = 4;
    pub const GPS: u8 = 5;
    //.....
    pub const GS: u8 = 7;
    pub const COMS: u8 = 8;
}


/// For constants that are used across the entire project
pub mod constants {
    pub const UHF_MAX_MESSAGE_SIZE_BYTES: u8 = 128;
}


/// Here opcodes and their associated meaning are defined for each component 
/// This is in common lib because components will need to know what opcodes to use when sending messages to other components
/// For example if a message is sent to the OBC to get housekeeping data, 
pub mod opcodes {
    pub mod coms{
        pub const GET_HK: u8 = 3;
        pub const SET_BEACON: u8 = 4;
        pub const GET_BEACON: u8 = 5;
    }
    pub mod dfgm {
        pub const TOGGLE_DATA_COLLECTION: u8 = 0;
    }
}