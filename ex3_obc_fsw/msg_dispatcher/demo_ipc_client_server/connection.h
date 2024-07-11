
/*
 * File connection.h
 */

#ifndef CONNECTION_H
#define CONNECTION_H

#define SOCKET_PATH_PREPEND "/tmp/fifo_socket_num_"
#define BUFFER_SIZE 32

void handle_error(char *error_msg);
int create_socket();

#endif
