pub mod component_ids;
pub use component_ids::{ComponentIds};
pub mod message_structure;
pub mod bulk_msg_slicing;
pub mod logging;

/// Ports used for communication between handlers and simulated subsystems / payloads
pub mod ports {
    pub const SIM_DFGM_PORT: u16 = 1802;
    pub const SIM_ADCS_PORT: u16 = 1803;
    pub const SIM_EPS_PORT: u16 = 1804;
    pub const SIM_ESAT_UART_PORT: u16 = 1805;
    pub const SIM_IRIS_PORT: u16 = 1806;
    pub const SIM_ESAT_UHF_PORT: u16 = 1808;
    pub const SIM_ESAT_BEACON_PORT: u16 = 1809;

    pub const DFGM_HANDLER_DISPATCHER_PORT: u16 = 1900;
    pub const SCHEDULER_DISPATCHER_PORT: u16 = 1901;
    pub const SUBSYSTEM_MONITOR_DISPATCHER_PORT: u16 = 1902;
    pub const BULK_MSG_HANDLER_DISPATCHER_PORT: u16 = 1903;
}

/// For constants that are used across the entire project
pub mod constants {
    pub const UHF_MAX_MESSAGE_SIZE_BYTES: u8 = 128;

    // Something up with the slicing makes this number be the size that each packet ends up 128B
    pub const DONWLINK_MSG_BODY_SIZE: usize = 121; // 128 - 5 (header) - 2 (sequence number)
}

/// Here opcodes and their associated meaning are defined for each component
/// This is in common lib because components will need to know what opcodes to use when sending messages to other components
/// For example if a message is sent to the OBC to get housekeeping data,
pub mod opcodes {
    pub enum COMS {
        GetHK = 3,
        SetBeacon = 4,
        GetBeacon = 5,
        Error = 6,
    }
    pub enum DFGM {
        ToggleDataCollection = 0,
        Error = 99,
    }

    // For IRIS subsystem
    pub enum IRIS {
        CaptureImage = 0,
        ToggleSensor = 1,
        FetchImage = 2,
        GetHK = 3,
        GetNImagesAvailable = 4,
        SetTime = 5,
        GetTime = 6,
        Reset = 7,
        DelImage = 8,
        GetImageSize = 9,
        Error = 99,
    }

    pub enum UHF {
        GetHK = 3,
        SetBeacon = 4,
        GetBeacon = 5,
        SetMode = 6,
        Reset = 7,
        GetMode = 8,
        Error = 99,
    }

    impl From<u8> for COMS {
        fn from(value: u8) -> Self {
            match value {
                3 => COMS::GetHK,
                4 => COMS::SetBeacon,
                5 => COMS::GetBeacon,
                _ => {
                    COMS::Error // or choose a default value or handle the error in a different way
                }
            }
        }
    }

    impl From<u8> for DFGM {
        fn from(value: u8) -> Self {
            match value {
                0 => DFGM::ToggleDataCollection,
                _ => {
                    DFGM::Error // or choose a default value or handle the error in a different way
                }
            }
        }
    }

    impl From<u8> for IRIS {
        fn from(value: u8) -> Self {
            match value {
                0 => IRIS::CaptureImage,
                1 => IRIS::ToggleSensor,
                2 => IRIS::FetchImage,
                3 => IRIS::GetHK,
                4 => IRIS::GetNImagesAvailable,
                5 => IRIS::SetTime,
                6 => IRIS::GetTime,
                7 => IRIS::Reset,
                8 => IRIS::DelImage,
                9 => IRIS::GetImageSize,
                _ => {
                    IRIS::Error // or choose a default value or handle the error in a different way
                }
            }
        }
    }

    impl From<u8> for UHF {
        fn from(value: u8) -> Self {
            match value {
                3 => UHF::GetHK,
                4 => UHF::SetBeacon,
                5 => UHF::GetBeacon,
                6 => UHF::SetMode,
                7 => UHF::Reset,
                8 => UHF::GetMode,
                _ => {
                    UHF::Error // or choose a default value or handle the error in a different way
                }
            }
        }
    }

    pub enum ADCS {
        Detumble = 0,
        OnOff = 1,
        WheelSpeed = 2,
        GetHk = 3,
        MagnetorquerCurrent = 4,
        OnboardTime = 5,
        GetOrientation = 6,
        Reset = 7,
        OrientToSBand = 9,
        Error = 99,
    }
    impl From<u8> for ADCS {
        fn from(value: u8) -> Self {
            match value {
                0 => ADCS::Detumble,
                1 => ADCS::OnOff,
                2 => ADCS::WheelSpeed,
                3 => ADCS::GetHk,
                4 => ADCS::MagnetorquerCurrent,
                5 => ADCS::OnboardTime,
                6 => ADCS::GetOrientation,
                7 => ADCS::Reset,
                9 => ADCS::OrientToSBand,
                _ => {
                    eprintln!("Invalid opcode: {}", value);
                    ADCS::Error
                }
            }
        }
    }
    impl std::fmt::Display for ADCS {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match *self {
                ADCS::Detumble => write!(f, "Detumble"),
                ADCS::OnOff => write!(f, "On/Off"),
                ADCS::WheelSpeed => write!(f, "Wheel Speed"),
                ADCS::GetHk => write!(f, "Get Housekeeping"),
                ADCS::MagnetorquerCurrent => write!(f, "Magnetorquer Current"),
                ADCS::OnboardTime => write!(f, "Onboard Time"),
                ADCS::Reset => write!(f, "Reset"),
                ADCS::GetOrientation => write!(f, "Get Orientation"),
                ADCS::OrientToSBand => write!(f, "Orient to S-Band"),
                ADCS::Error => write!(f, "INVALID OPCODE"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::component_ids;

    #[test]
    fn get_component_enum_from_u8() {
        //Test conversion from u8 to ComponentIds enum for all
        let eps = component_ids::ComponentIds::try_from(1).unwrap();
        assert_eq!(eps, component_ids::ComponentIds::EPS);

        let adcs = component_ids::ComponentIds::try_from(2).unwrap();
        assert_eq!(adcs, component_ids::ComponentIds::ADCS);

        let dfgm = component_ids::ComponentIds::try_from(3).unwrap();
        assert_eq!(dfgm, component_ids::ComponentIds::DFGM);

        let iris = component_ids::ComponentIds::try_from(4).unwrap();
        assert_eq!(iris, component_ids::ComponentIds::IRIS);

        let gps = component_ids::ComponentIds::try_from(5).unwrap();
        assert_eq!(gps, component_ids::ComponentIds::GPS);

        let deployables = component_ids::ComponentIds::try_from(6).unwrap();
        assert_eq!(deployables, component_ids::ComponentIds::DEPLOYABLES);

        let gs = component_ids::ComponentIds::try_from(7).unwrap();
        assert_eq!(gs, component_ids::ComponentIds::GS);

        let coms = component_ids::ComponentIds::try_from(8).unwrap();
        assert_eq!(coms, component_ids::ComponentIds::COMS);

        let shell = component_ids::ComponentIds::try_from(10).unwrap();
        assert_eq!(shell, component_ids::ComponentIds::SHELL);

        let uhf = component_ids::ComponentIds::try_from(11).unwrap();
        assert_eq!(uhf, component_ids::ComponentIds::UHF);

        let obc = component_ids::ComponentIds::try_from(0).unwrap();
        assert_eq!(obc, component_ids::ComponentIds::OBC);
    }

    #[test]
    fn get_component_enum_from_str() {
        let eps = component_ids::ComponentIds::from_str("EPS").unwrap();
        assert_eq!(eps, component_ids::ComponentIds::EPS);

        let adcs = component_ids::ComponentIds::from_str("ADCS").unwrap();
        assert_eq!(adcs, component_ids::ComponentIds::ADCS);

        let dfgm = component_ids::ComponentIds::from_str("DFGM").unwrap();
        assert_eq!(dfgm, component_ids::ComponentIds::DFGM);

        let iris = component_ids::ComponentIds::from_str("IRIS").unwrap();
        assert_eq!(iris, component_ids::ComponentIds::IRIS);

        let gps = component_ids::ComponentIds::from_str("GPS").unwrap();
        assert_eq!(gps, component_ids::ComponentIds::GPS);

        let deployables = component_ids::ComponentIds::from_str("DEPLOYABLES").unwrap();
        assert_eq!(deployables, component_ids::ComponentIds::DEPLOYABLES);

        let gs = component_ids::ComponentIds::from_str("GS").unwrap();
        assert_eq!(gs, component_ids::ComponentIds::GS);

        let coms = component_ids::ComponentIds::from_str("COMS").unwrap();
        assert_eq!(coms, component_ids::ComponentIds::COMS);

        let shell = component_ids::ComponentIds::from_str("SHELL").unwrap();
        assert_eq!(shell, component_ids::ComponentIds::SHELL);

        let uhf = component_ids::ComponentIds::from_str("UHF").unwrap();
        assert_eq!(uhf, component_ids::ComponentIds::UHF);

        let obc = component_ids::ComponentIds::from_str("OBC").unwrap();
        assert_eq!(obc, component_ids::ComponentIds::OBC);
    }

    #[test]
    fn get_component_str_from_enum() {
        let eps = component_ids::ComponentIds::EPS;
        assert_eq!(eps.to_string(), "EPS");

        let adcs = component_ids::ComponentIds::ADCS;
        assert_eq!(adcs.to_string(), "ADCS");

        let dfgm = component_ids::ComponentIds::DFGM;
        assert_eq!(dfgm.to_string(), "DFGM");

        let iris = component_ids::ComponentIds::IRIS;
        assert_eq!(iris.to_string(), "IRIS");

        let gps = component_ids::ComponentIds::GPS;
        assert_eq!(gps.to_string(), "GPS");

        let deployables = component_ids::ComponentIds::DEPLOYABLES;
        assert_eq!(deployables.to_string(), "DEPLOYABLES");

        let gs = component_ids::ComponentIds::GS;
        assert_eq!(gs.to_string(), "GS");

        let coms = component_ids::ComponentIds::COMS;
        assert_eq!(coms.to_string(), "COMS");

        let uhf = component_ids::ComponentIds::UHF;
        assert_eq!(uhf.to_string(), "UHF");

        let obc = component_ids::ComponentIds::OBC;
        assert_eq!(obc.to_string(), "OBC");
    }
}
