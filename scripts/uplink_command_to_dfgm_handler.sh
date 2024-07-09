#!/usr/bin/bash

# This script runs the first leg of our 'tall and thin' implementation of software architecture. 
#  The script sends a msg (command), from the gs (right now its mocked as a tcp server), up to the UHF transceiver, 
#  which sends the msg to the msg dispatcher (based on msg destination id), which routes it to the DFGM handler based on the msg destination ID.  

## Launch the GS simulation (this can just be a tcp server for now )
gnome-terminal -t SIM_GS -- sh -c 'cd /home/devin/albertasat/ex3/gs/cli_ground_station/cli_ground_station/ && cargo run ; bash exec'

## Create the simulated subystem components (dfgm and uhf transciever) - because they are tcp servers  
gnome-terminal -t SIM_DFGM_SUBSYSTEM -- sh -c 'cd /home/devin/albertasat/ex3/sim_subs/ex3_simulated_subsystems/DFGM && python3 ./dfgm_subsystem.py ; bash exec;'

# For now the UHF transceiver is bypassed and the GS sends msgs directly to the coms handler 

## Create the msg dispatcher (first component of the obc fsw because it creates ipc servers 
gnome-terminal -t MSG_DISPATCHER -- sh -c 'cd ../msg_dispatcher && ./msg_dispatcher; exec bash'
sleep 0.25

## Create the hanlders and other obc fsw components (coms handler, dfgm handler )

#TODO - instantiate the DFGM handler 
gnome-terminal -t COMS_HANDLER -- sh -c 'cd ../ && cargo run --bin coms_handler; exec bash'
