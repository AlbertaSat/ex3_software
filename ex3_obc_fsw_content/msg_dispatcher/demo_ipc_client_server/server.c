/*
TODO
 How to handle if an arch component connection is lost with the dispatcher ?
 How to handle if an arch component is not being used (can be ignored by dispatcher) i.e. for testing or if it needs to be killed and restarted ?
*/

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>
#include <poll.h>
#include <fcntl.h>
#include "connection.h"

#define LISTEN_BACKLOG_SIZE 3

int main(int argc, char *argv[])
{
    // Number of sockets to listen on defined as first arg

    if (argc != 2)
    {
        fprintf(stderr, "Usage: %s <num_sockets>\n", argv[0]);
        exit(EXIT_FAILURE);
    }

    int num_sockets = atoi(argv[1]);

    if (num_sockets < 1)
    {
        fprintf(stderr, "Number of sockets must be greater than 0\n");
        exit(EXIT_FAILURE);
    }

    // Create an array of sockets
    struct sockaddr_un *s_name = calloc(num_sockets, sizeof(struct sockaddr_un));
    int con_sockets[num_sockets];  // fd for socket listening for connections
    int data_sockets[num_sockets]; // fd for sockets that have a connection
    int ret;                       // return value used for various POSIX system calls and libc fxns
    char buffer[BUFFER_SIZE];      // For storing the message read from a socket during a POLLIN revent
    nfds_t nfds;                   // Num file descriptors (how many fds are we polling)
    ssize_t s;
    struct pollfd *pfds;
    nfds = num_sockets;
    pfds = calloc(nfds, sizeof(struct pollfd));
    int ready;                       // How many sockets are ready for polling
    int closed_data_socket_num = -1; // Flag set to a fd value when the associated fd client connection is dropped
    int dest_id = -1;                // The destination ID of a recevied message (for now just first byte of input)

    for (int i = 0; i < num_sockets; i++)
    {
        /* Create a socket struct for listening for connection */
        con_sockets[i] = create_socket();

        memset(&s_name[i], 0, sizeof(s_name));

        s_name[i].sun_family = AF_UNIX;
        char fifo_name[32];
        // Create a name of the fifo based on this iter val
        sprintf(fifo_name, "%s%d", SOCKET_PATH_PREPEND, i);
        printf("Created Fifo: %s\n", fifo_name);

        // Copy the name of the socket (fifo abs path) into the socket name struct
        strncpy(s_name[i].sun_path, fifo_name, sizeof(s_name[i].sun_path) - 1);
        unlink(s_name[i].sun_path); // Called before bind to remove socket it if already exists

        /* Bind to the socket */
        ret = bind(con_sockets[i], (const struct sockaddr *)&s_name[i],
                   sizeof(s_name[i]));
        if (ret == -1)
        {
            perror("bind");
            exit(EXIT_FAILURE);
        }

        //  * Prepare for accepting connections. The backlog size is set
        //  * to 20. So while one request is being processed other requests
        //  * can be waiting.
        //  */
        ret = listen(con_sockets[i], LISTEN_BACKLOG_SIZE);
        if (ret == -1)
        {
            perror("listen");
            exit(EXIT_FAILURE);
        }
    }

    /* Blocking accept for expected number of client connections */
    for (int i = 0; i < num_sockets; i++)
    {
        data_sockets[i] = accept(con_sockets[i], NULL, NULL);
        if (data_sockets[i] == -1)
        {
            perror("accept");
            exit(EXIT_FAILURE);
        }
        pfds[i].fd = data_sockets[i];
        pfds[i].events = POLLIN;
    }

    for (;;)
    {
        /* Wait for incoming connection. */
        while (closed_data_socket_num == -1)
        {
            printf("\nAbout to poll()\n");
            ready = poll(pfds, nfds, -1);
            if (ready == -1)
                perror("poll");
            printf("Ready: %d\n", ready);

            for (nfds_t i = 0; i < nfds; i++)
            {
                if (pfds[i].revents != 0)
                {
                    if (pfds[i].revents & POLLIN)
                    {
                        s = read(pfds[i].fd, buffer, sizeof(buffer));
                        if (s == -1)
                        {
                            perror("read");
                            exit;
                        }
                        if (s == 0)
                        {
                            closed_data_socket_num = i;
                            printf("Connection to socket: %d closed . {zero byte read indicates this}\n", (int)i);
                            exit;
                        }
                        printf("read %zd bytes: %.*s", s, (int)s, buffer);

                        // First byte is the destination socket num
                        dest_id = buffer[0] - '0'; // convert ascii char to int equivalent ('0' is 48)
                        printf("Destination ID field value: %d \n", dest_id);
                        if (dest_id > -1 && dest_id < num_sockets)
                        {
                            s = write(pfds[dest_id].fd, buffer, strlen(buffer) + 1);
                            if (s > 0)
                            {
                                printf("Wrote: %d bytes to socket num: %d \n", (int)s, dest_id);
                            }
                        }
                    }
                    else
                    { /* POLLERR | POLLHUP */
                        printf("    closing fd %d\n", pfds[i].fd);
                        // get the value of the fd for the closed data socket -
                        closed_data_socket_num = i;
                        if (close(pfds[i].fd) == -1)
                            perror("closing socket");
                        exit;
                    }
                }
            }
        }

        // TODO - Add a timeout for this
        printf("Listening - blocking wait for for connection request from client num: %d\n", closed_data_socket_num);
        data_sockets[closed_data_socket_num] = accept(con_sockets[closed_data_socket_num], NULL, NULL);
        if (data_sockets[closed_data_socket_num] == -1)
        {
            perror("accept");
            exit(EXIT_FAILURE);
        }
        pfds[closed_data_socket_num].fd = data_sockets[closed_data_socket_num];
        pfds[closed_data_socket_num].events = POLLIN;
        closed_data_socket_num = -1; // reset flag now that connection re-established
    }

    exit(EXIT_SUCCESS);
}