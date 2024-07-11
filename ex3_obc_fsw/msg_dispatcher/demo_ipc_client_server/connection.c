#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>
#include <poll.h>
#include <fcntl.h>
#include "connection.h"

void handle_error(char *error_msg){
       fprintf(stderr, "Error: %s", error_msg); 
       exit(EXIT_FAILURE);
}

int create_socket(){
    int data_socket = socket(AF_UNIX, SOCK_SEQPACKET, 0);
    if (data_socket == -1){
        handle_error("socket_creation");
    }
    return data_socket; 
}