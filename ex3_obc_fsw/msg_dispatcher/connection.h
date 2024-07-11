
/*
Written by Devin Headrick
Summer 2024
*/

#ifndef CONNECTION_H
#define CONNECTION_H

#define MSG_UNIT_SIZE 128
#define LISTEN_BACKLOG_SIZE 3
#define SOCKET_PATH_PREPEND "/tmp/fifo_socket_"
#define COMPONENT_NAME_SIZE_MAX 32
#define FIFO_PATH_SIZE_MAX 64

// Based of this google sheet : https://docs.google.com/spreadsheets/d/1rWde3jjrgyzO2fsg2rrVAKxkPa2hy-DDaqlfQTDaNxg/edit?gid=0#gid=0
// For most cases messages destinated for a particular payload / subsystem are passed to its associated handler.
enum ComponentId
{
    OBC = 0,
    EPS = 1,
    ADCS = 2,
    DFGM = 3,
    IRIS = 4,
    GPS = 5,
    DEPLOYABLES = 6,
    GS = 7,
    COMS = 8,
    TEST = 99,
};

/// @brief Each architecture component the message dispatcher will talk to has its own struct
typedef struct ComponentStruct
{
    char name[COMPONENT_NAME_SIZE_MAX]; // Name of subsystem / payload
    enum ComponentId component_id;      // The enumerated ID of the subsystem/payload associated with a message
    char fifo_path[FIFO_PATH_SIZE_MAX]; // Path to fifo used by Unix Domain Socket
    int connected;                      // connection state, 0 = not conn, 1 = connected
    int conn_socket_fd;                 // connection socket fd for polling for connection
    int data_socket_fd;                 // data socket fd for polling when there is a connection
    struct sockaddr_un conn_socket;
    struct sockaddr_un data_socket;
} ComponentStruct;


ComponentStruct *component_factory(char *name, int component_id);

int get_msg_dest_id(char *data_buf);
void handle_error(char *error_msg);
int create_socket();
int get_fd_from_id(ComponentStruct *cs[], int num_components, int id);
void accept_incoming_client_conn_request(ComponentStruct *component, struct pollfd *poll_struct);
int read_data_socket(ComponentStruct *component, struct pollfd *poll_struct, char *buffer);

#endif