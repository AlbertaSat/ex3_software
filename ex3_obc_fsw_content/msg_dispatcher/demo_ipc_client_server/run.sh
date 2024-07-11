#!/usr/bin/bash

CLIENT_COUNT=$1
# Create the server with provided number of sockets awaiting client connection
gnome-terminal -t SERVER --working-directory $PWD -- sh -c "./server ${CLIENT_COUNT}; exec bash"

# Create a new terminal for each client 
for ((c=0; c<$CLIENT_COUNT; c++))
do 
    gnome-terminal -t "CLIENT ${c}" --working-directory $PWD -- sh -c "./client ${c}; exec bash"
done 