/// Ports used for communication between handlers and simulated subsystems / payloads
pub mod ports {
    pub const SIM_DFGM_PORT: u16 = 1802;
    pub const SIM_ADCS_PORT: u16 = 1803;
    pub const SIM_EPS_PORT: u16 = 1804;
    pub const SIM_COMMS_PORT: u16 = 1805;
    pub const SIM_IRIS_PORT: u16 = 1806;
    pub const SIM_DUMMY_PORT: u16 = 1807;
    pub const SIM_UHF_GS_PORT: u16 = 1808;

    pub const DFGM_HANDLER_DISPATCHER_PORT: u16 = 1900;
    pub const SCHEDULER_DISPATCHER_PORT: u16 = 1901;
    pub const SUBSYSTEM_MONITOR_DISPATCHER_PORT: u16 = 1902;
    pub const BULK_MSG_HANDLER_DISPATCHER_PORT: u16 = 1903;
}

/// Each thing that can emit or receive a message has an associated ID. Each message header includes this id for source and destination.
/// Referencing this page:
pub mod component_ids {
    use std::fmt;
    use std::str::FromStr;

    // ---------- Depricated but left to not break things -------- //
    pub const OBC: u8 = 0;
    pub const EPS: u8 = 1;
    pub const ADCS: u8 = 2;
    pub const DFGM: u8 = 3;
    pub const IRIS: u8 = 4;
    pub const GPS: u8 = 5;
    //.....
    pub const GS: u8 = 7;
    pub const COMS: u8 = 8;
    // ----------------------------------------------------------- //

    #[derive(PartialEq, Debug)]
    pub enum ComponentIds {
        OBC = 0,
        EPS = 1,
        ADCS = 2,
        DFGM = 3,
        IRIS = 4,
        GPS = 5,
        //...
        GS = 7,
        COMS = 8,
        BulkMsgDispatcher = 9,
        //..
        CMD = 10,
        //..
        DUMMY = 99,
    }

    impl fmt::Display for ComponentIds {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                ComponentIds::OBC => write!(f, "OBC"),
                ComponentIds::EPS => write!(f, "EPS"),
                ComponentIds::ADCS => write!(f, "ADCS"),
                ComponentIds::DFGM => write!(f, "DFGM"),
                ComponentIds::IRIS => write!(f, "IRIS"),
                ComponentIds::GPS => write!(f, "GPS"),
                ComponentIds::GS => write!(f, "GS"),
                ComponentIds::COMS => write!(f, "COMS"),
                ComponentIds::BulkMsgDispatcher => write!(f, "BulkMsgDispatcher"),
                ComponentIds::CMD => write!(f, "CMD"),
                ComponentIds::DUMMY => write!(f, "DUMMY"),
            }
        }
    }
    impl FromStr for ComponentIds {
        type Err = ();
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "OBC" => Ok(ComponentIds::OBC),
                "EPS" => Ok(ComponentIds::EPS),
                "ADCS" => Ok(ComponentIds::ADCS),
                "DFGM" => Ok(ComponentIds::DFGM),
                "IRIS" => Ok(ComponentIds::IRIS),
                "GPS" => Ok(ComponentIds::GPS),
                "GS" => Ok(ComponentIds::GS),
                "COMS" => Ok(ComponentIds::COMS),
                "BulkMsgDispatcher" => Ok(ComponentIds::BulkMsgDispatcher),
                //...
                "CMD" => Ok(ComponentIds::CMD),
                "DUMMY" => Ok(ComponentIds::DUMMY),
                _ => Err(()),
            }
        }
    }

    //TODO - Find a way to make this return a result type instead of panicking
    //       - the 'From<u8> method from std::convert lib does not allow for returning a Result type
    impl TryFrom<u8> for ComponentIds {
        type Error = ();

        fn try_from(value: u8) -> Result<Self, Self::Error> {
            match value {
                x if x == ComponentIds::OBC as u8 => Ok(ComponentIds::OBC),
                x if x == ComponentIds::EPS as u8 => Ok(ComponentIds::EPS),
                x if x == ComponentIds::ADCS as u8 => Ok(ComponentIds::ADCS),
                x if x == ComponentIds::DFGM as u8 => Ok(ComponentIds::DFGM),
                x if x == ComponentIds::IRIS as u8 => Ok(ComponentIds::IRIS),
                x if x == ComponentIds::GPS as u8 => Ok(ComponentIds::GPS),
                x if x == ComponentIds::GS as u8 => Ok(ComponentIds::GS),
                x if x == ComponentIds::COMS as u8 => Ok(ComponentIds::COMS),
                x if x == ComponentIds::BulkMsgDispatcher as u8 => Ok(ComponentIds::BulkMsgDispatcher),
                x if x == ComponentIds::CMD as u8 => Ok(ComponentIds::CMD),
                x if x == ComponentIds::DUMMY as u8 => Ok(ComponentIds::DUMMY),
                _ => Err(()),
            }
        }
    }
}

/// For constants that are used across the entire project
pub mod constants {
    pub const UHF_MAX_MESSAGE_SIZE_BYTES: u8 = 128;
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

    // For dummy subsystem - used in testing and development
    pub enum DUMMY {
        SetDummyVariable = 0,
        GetDummyVariable = 1,
    } 

    impl From<u8> for DUMMY {
        fn from(value: u8) -> Self {
            match value {
                0 => DUMMY::SetDummyVariable,
                1 => DUMMY::GetDummyVariable,
                _ => {
                    eprintln!("Invalid opcode: {}", value);
                    DUMMY::GetDummyVariable // or choose a default value or handle the error in a different way
                }
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

        let gs = component_ids::ComponentIds::try_from(7).unwrap();
        assert_eq!(gs, component_ids::ComponentIds::GS);

        let coms = component_ids::ComponentIds::try_from(8).unwrap();
        assert_eq!(coms, component_ids::ComponentIds::COMS);

        let test = component_ids::ComponentIds::try_from(99).unwrap();
        assert_eq!(test, component_ids::ComponentIds::DUMMY);

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

        let gs = component_ids::ComponentIds::from_str("GS").unwrap();
        assert_eq!(gs, component_ids::ComponentIds::GS);

        let coms = component_ids::ComponentIds::from_str("COMS").unwrap();
        assert_eq!(coms, component_ids::ComponentIds::COMS);

        let test = component_ids::ComponentIds::from_str("DUMMY").unwrap();
        assert_eq!(test, component_ids::ComponentIds::DUMMY);

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

        let gs = component_ids::ComponentIds::GS;
        assert_eq!(gs.to_string(), "GS");

        let coms = component_ids::ComponentIds::COMS;
        assert_eq!(coms.to_string(), "COMS");

        let test = component_ids::ComponentIds::DUMMY;
        assert_eq!(test.to_string(), "DUMMY");

        let obc = component_ids::ComponentIds::OBC;
        assert_eq!(obc.to_string(), "OBC");
    }
}
