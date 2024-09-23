#!/usr/bin/bash

CLIENT_COUNT=$1

# Create a detached session using our config file to hold our windows
tmux -f ../../../scripts/.tmux.conf new-session -d -s "demo_ipc_client_server"

# Create the server with provided number of sockets awaiting client connection
tmux new-window -n "SERVER" -c $PWD -- "trap : SIGINT; ./server ${CLIENT_COUNT}; exec bash"
#                                       ^ to continue after CTRL+C

# Create a new terminal for each client 
for ((c=0; c<$CLIENT_COUNT; c++))
do 
    tmux new-window -n "CLIENT ${c}" -c $PWD -- "trap : SIGINT; ./client ${c}; exec bash"
done 

tmux attach-session -t "demo_ipc_client_server"
