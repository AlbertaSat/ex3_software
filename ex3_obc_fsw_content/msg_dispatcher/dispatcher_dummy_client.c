/*
 * File client.c
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

#define NUM_CLIENTS_TO_POLL 2 //to poll both the unix domain socket fd AND stdin fd
#define CLIENT_POLL_TIMEOUT_MS 10

int main(int argc, char *argv[])
{
    if (argc != 2)
    {
        fprintf(stderr, "Usage: %s <client_id>\n", argv[0]);
        exit(EXIT_FAILURE);
    }

    char *client_id = argv[1];

    char fifo_name[MSG_UNIT_SIZE];
    // Arg 1 is the client 'num' which determines what named pipe to associated with the socket
    sprintf(fifo_name, "%s%s", SOCKET_PATH_PREPEND, client_id);

    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    int ret;
    int data_socket;
    char socket_buf[MSG_UNIT_SIZE];
    char std_in_buf[MSG_UNIT_SIZE];
    int ready;

    // Setup polling for non blocking read
    struct sockaddr_un *s_name = calloc(NUM_CLIENTS_TO_POLL, sizeof(struct sockaddr_un));
    nfds_t nfds;
    struct pollfd *pfds;
    nfds = NUM_CLIENTS_TO_POLL;
    pfds = calloc(nfds, sizeof(struct pollfd));
    
    // Poll from stdin as well as the socket (for now clients take user input and send this as message)
    pfds[0].fd = 0;
    pfds[0].events = POLLIN;

    data_socket = create_socket();

    printf("Attempting to connect to %s\n", fifo_name);

    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, fifo_name, sizeof(addr.sun_path) - 1);

    ret = connect(data_socket, (const struct sockaddr *)&addr,
                  sizeof(addr));
    if (ret == -1)
    {
        handle_error("The server is down\n");
    }
    printf("Successfully Connected to %s\n", fifo_name);

    pfds[1].fd = data_socket;
    pfds[1].events = POLLIN;
    
    while (1)
    {
        ready = poll(pfds, nfds, CLIENT_POLL_TIMEOUT_MS); // 10 milli sec timeout
        if (ready == -1)
        {
            perror("poll");
            exit;
        }
        for (nfds_t i = 0; i < nfds; i++)
        {
            if (pfds[i].revents != 0)
            {
                if (pfds[i].revents & POLLIN)
                {
                    if (i == 0)
                    { // if std in
                        printf("reading from std in: \n");
                        fgets(std_in_buf, MSG_UNIT_SIZE, stdin);

                        ret = write(data_socket, std_in_buf, strlen(std_in_buf) + 1);
                        if (ret == -1)
                        {
                            perror("write");
                            break;
                        }
                    }
                    else
                    {
                        ret = read(data_socket, socket_buf, sizeof(socket_buf));
                        if (ret == -1)
                        {
                            perror("read");
                            exit(EXIT_FAILURE);
                        }
                        else if (ret == 0)
                        {
                            printf("Connection to server droppped. Exiting... \n");
                            return 0;
                        }
                        else if(ret > 0)
                        {
                            printf("Received: %s", socket_buf);
                        }
                    }
                }
            }
        }
    }
    exit(EXIT_SUCCESS);
}