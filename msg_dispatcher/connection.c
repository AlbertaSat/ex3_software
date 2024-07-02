/*
Written By Devin Headrick
Summer 2024
*/

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

/// @brief If errno is not set by a function (perror cannot be used), then this fxn is used  to handle errors that crash the program
/// @param error_msg
void handle_error(char *error_msg)
{
    fprintf(stderr, "Error: %s", error_msg);
    exit(EXIT_FAILURE);
}

/// @brief Create a Unix Domain socket using SOCK_SEQPACKET type sockets, which have defined boundaries (a message 'unit' rather than stream)
/// @return
int create_socket()
{
    int data_socket = socket(AF_UNIX, SOCK_SEQPACKET, 0);
    if (data_socket == -1)
    {
        handle_error("socket_creation");
    }
    return data_socket;
}

/// @brief Use byte offset of a read msg to get destination ID.
/// @param data_buf
/// @return
int get_msg_dest_id(char *data_buf)
{
    // Use byte offset
<<<<<<< HEAD
<<<<<<< HEAD
    int dest_id = data_buf[2];
=======
    int dest_id = data_buf[46];

    // TEMP for now we are sending ascii chars, so we want to offset the numerical value
    dest_id -= '0';
>>>>>>> 0aaadca (Very beta working tall-thin with json serdes. ipc_dummy_client acts as TCP server for input.)
=======
    int dest_id = data_buf[2];
>>>>>>> 6a33a7f (Serdes now uses custom functions to have no overhead. Also changed connection.c to not change byte value read. TALLTHIN WORKS)
    printf("Msg Dest ID: %d\n", dest_id);
    return dest_id;
}

/// @brief Constructor that creates a component struct by malloc-ing space required and assigning default values
/// @param name
/// @param component_id
/// @return *ComponentStruct
ComponentStruct *component_factory(char *name, int component_id)
{
    int ret;
    ComponentStruct *c = malloc(sizeof(ComponentStruct));
    strcpy(c->name, name);
    c->component_id = component_id;
    c->connected = 0;
    // 1. Create a socket and get its associated fd
    c->conn_socket_fd = create_socket();
    printf("Created conn socket, with fd: %d\n", c->conn_socket_fd);

    // 2. Bind the conn socket fd [from socket() call] to an address in unix domain (fifo file)
    sprintf(c->fifo_path, "%s%s", SOCKET_PATH_PREPEND, c->name);
    strcpy(c->conn_socket.sun_path, c->fifo_path); // strcpy path into conn socket addr
    printf("Socket file path: %s\n", c->conn_socket.sun_path);
    unlink(c->conn_socket.sun_path); // remove socket if it already exists
    c->conn_socket.sun_family = AF_UNIX;
    ret = bind(c->conn_socket_fd, (const struct sockaddr *)&c->conn_socket, sizeof(c->conn_socket));
    if (ret < 0)
    {
        handle_error("binding conn socket\n");
    }
    printf("Bind conn socket for %s \n", name);

    // 3. Call listen on the bound socket for incomming conn requests from clients
    ret = listen(c->conn_socket_fd, LISTEN_BACKLOG_SIZE);
    if (ret == -1)
    {
        handle_error("conn socket listen\n");
    }
    printf("Listening conn socket for %s \n", name);
    // 4. Call accept to accept conn request from client - returns new descriptor for data between client and server
    // 5. Handle connection now with data socket

    return c;
}

/// @brief Get the file descriptor associated with a component based on its component ID
/// @param cs
/// @param num_components
/// @param id
/// @return Associated fd for that component
int get_fd_from_id(ComponentStruct *cs[], int num_components, int id)
{
    // loop over components, get fd of one with matching id
    for (int i = 0; i < num_components; i++)
    {
        if (cs[i]->component_id == id)
        {
            printf("Component match : %s, with id: %d \n", cs[i]->name, cs[i]->component_id);

            // if connected flag is low (component not connected) then return -1
            if (cs[i]->connected == 0)
            {
                printf("Component not connected. Not writing \n");
                return -2;
            }
            printf("Destination component fd: %d \n", id);
            return cs[i]->data_socket_fd;
        }
    }
    printf("No matching component found with id: %d \n", id);
    return -1;
}

/// @brief Accept an incoming client request. Update associated component structs data fd and connected flag
/// @param component
/// @param poll_struct
void accept_incoming_client_conn_request(ComponentStruct *component, struct pollfd *poll_struct)
{
    socklen_t addrlen = sizeof(component->conn_socket);
    int ret = accept(component->conn_socket_fd, (struct sockaddr *)&component->data_socket, &addrlen);
    if (ret == -1)
    {
        perror("accept");
        exit(EXIT_FAILURE);
    }
    component->data_socket_fd = ret;
    printf("%s data socket val: %d\n", component->name, component->data_socket_fd);
    poll_struct->fd = component->data_socket_fd; // set the pfd for the associated component to use the fd associated with the data socket
    component->connected = 1;
}

/// @brief Read the incomming data from a connected component. If zero is read, then we know conn is dropped and flag is zeroed
/// @param component
/// @param poll_struct
/// @param buffer
/// @return Number of bytes read
int read_data_socket(ComponentStruct *component, struct pollfd *poll_struct, char *buffer)
{
    int ret = read(poll_struct->fd, buffer, MSG_UNIT_SIZE);
    if (ret == -1)
    {
        perror("read");
        exit(EXIT_FAILURE);
    }
    else if (ret == 0)
    {
        poll_struct->fd = component->conn_socket_fd; // Go back to polling the conn socket fd to listen for client connections components[i]->connected = 0;               // Reset the conn flag (so we know we are back to looking for conn revents on the poll)
        component->connected = 0;
        printf("Connection to socket: %s closed . {zero byte read indicates this}\n", component->name);
        memset(buffer, 0, MSG_UNIT_SIZE);
    }
    else
    {
        printf("---------------------------------------\n");
        printf("Read %d bytes:\n", ret);
        //printf("Read data in ASCII: %.*s \n", ret, buffer);
        printf("Data in HEX is: \n");
        for (int i = 0; i < ret; i++)
        {
            printf(" %02x |", buffer[i]);
        }
        printf("\n---------------------------------------\n");
        return ret;
    }
}