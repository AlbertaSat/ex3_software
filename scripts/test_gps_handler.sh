#!/usr/bin/bash
# Written by Kaaden RumanCam
# Fall 2024

# User must provide the path to the simulated subsystem directory on their machine as the first arg
PATH_TO_SIM_SUBS=$1

if [ "$#" -lt 1 ]; then
    echo "ERROR=> Requires argument: <path to sim subsystem dir>\n"
    exit 0
fi;
echo "Path being used to sim subs: $PATH_TO_SIM_SUBS"

# Create a detached session using our config file to hold our windows
tmux -f .tmux.conf new-session -d -s "test_gps_handler"

# Launch the GPS simulator
tmux new-window -n "SIM_GPS" -- "trap : SIGINT; cd $PATH_TO_SIM_SUBS/GPS && python3 server.py; exec bash"

# Create bulk msg dispatcher
tmux new-window -n "BULK_MSG_DISPATCHER" -- "trap : SIGINT; cd ../ex3_obc_fsw/bulk_msg_dispatcher && cargo run; exec bash"
sleep 0.25

tmux new-window -n "SIM_UHF_SUBSYSTEM" -- "trap : SIGINT; cd $PATH_TO_SIM_SUBS/UHF && python3 ./simulated_uhf.py ; exec bash;"
sleep 0.25

tmux new-window -n "COMS_HANDLER" -- "trap : SIGINT; cd ../ && cargo run --bin coms_handler; exec bash"
sleep 0.25

## Launch the gps handler
tmux new-window -n "GPS_HANDLER" -- "trap : SIGINT; cd ../ && cargo run --bin gps_handler; exec bash"
sleep 0.25

## Create the msg dispatcher (last component of the obc fsw because it creates ipc clients
tmux new-window -n "CMD_DISPATCHER" -- "trap : SIGINT; cd ../ && cargo run --bin cmd_dispatcher; exec bash"
sleep 0.25

## Launch the GS simulation (this can just be a tcp server for now )
tmux new-window -n "SIM_GS" -- "trap : SIGINT; cd ../ && cargo run --bin cli_ground_station; exec bash"

tmux attach-session -t "test_gps_handler"
