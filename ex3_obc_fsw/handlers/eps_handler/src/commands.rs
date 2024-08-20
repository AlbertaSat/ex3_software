/*
Written by Devin Headrick
Summer 2024

This contains all the commands of interest relating to the EPS

Although commands that 'do stuff' with the EPS relative to the perspective of the OBC FSW and operator are found in the 'common' module, these are commands 
relating directly to the EPS hardware based on the Nanoavionics EPS User Manual.

Notes:
    - Binary commands - different 'types' of commands and interactions seem to have a 'binary commands' section in their docs
    - chapter 6 - application image - contains mission code and supports full set of subsystem features [FOR FIRMWARE FLASHES - This can wait]
    - chapter 7 - Ground station watchdog  
        for GSWDT - port 16 is used by default  
    - chap 8 - Logging

    There is a telemetry module onboard - two ways to get telem are: Send request get instantaneous telem, or download a telem file 

NOTE ALL REQUESTS AND REPLIES ARE LITTLE ENDIAN 

I think the easiest first command to implement will be the 'ping' because there is an example in the manual 

*/

//should be careful about this because most of the HK values can be retrieved from the instantenous telem return value
enum Commands {
    
}


enum CommandPorts {
    groundStationWatchdogTimer = 16, 
}

struct Command {
    port: CommandPorts
}


impl Command {
    fn new(port: CommandPorts) -> Self {
        Command {
            port: port
        }
    }
} 
