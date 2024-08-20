/*
Written by Devin Headrick
Summer 2024

*/
use std::fmt;

//TODO - on handler startup fetch the last stored hk values from non volatile mem  OR  fetch them from the EPS hardware directly again
#[derive(Debug, Clone, Copy)]
pub enum OutputChannelStates {
    Off,
    On,
}
