/// Each thing that can emit or receive a message has an associated ID. Each message header includes this id for source and destination.
/// Referencing this page:
use std::fmt::{self};
use std::str::FromStr;
use strum::EnumIter;

#[derive(EnumIter, PartialEq, Debug)]
pub enum ComponentIds {
    OBC = 0,
    EPS = 1,
    ADCS = 2,
    DFGM = 3,
    IRIS = 4,
    GPS = 5,
    DEPLOYABLES = 6,
    GS = 7,
    COMS = 8,
    BulkMsgDispatcher = 9,
    SHELL = 10,
    UHF = 11,
    LAST = 12,
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
            ComponentIds::DEPLOYABLES => write!(f, "DEPLOYABLES"),
            ComponentIds::GS => write!(f, "GS"),
            ComponentIds::COMS => write!(f, "COMS"),
            ComponentIds::BulkMsgDispatcher => write!(f, "BulkMsgDispatcher"),
            ComponentIds::SHELL => write!(f, "SHELL"),
            ComponentIds::UHF => write!(f, "UHF"),
            ComponentIds::LAST => write!(f, "illegal"),
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
            "DEPLOYABLES" => Ok(ComponentIds::DEPLOYABLES),
            "GS" => Ok(ComponentIds::GS),
            "COMS" => Ok(ComponentIds::COMS),
            "BulkMsgDispatcher" => Ok(ComponentIds::BulkMsgDispatcher),
            "SHELL" => Ok(ComponentIds::SHELL),
            "UHF" => Ok(ComponentIds::UHF),
            "LAST" => Err(()),
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
            x if x == ComponentIds::DEPLOYABLES as u8 => Ok(ComponentIds::DEPLOYABLES),
            x if x == ComponentIds::GS as u8 => Ok(ComponentIds::GS),
            x if x == ComponentIds::COMS as u8 => Ok(ComponentIds::COMS),
            x if x == ComponentIds::BulkMsgDispatcher as u8 => {
                Ok(ComponentIds::BulkMsgDispatcher)
            }
            x if x == ComponentIds::SHELL as u8 => Ok(ComponentIds::SHELL),
            x if x == ComponentIds::UHF as u8 => Ok(ComponentIds::UHF),
            x if x == ComponentIds::LAST as u8 => Err(()),
            _ => Err(()),
        }
    }
}
