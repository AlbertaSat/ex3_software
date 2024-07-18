/*
Written by Devin Headrick
Summer 2024

TODO - HANDLE THE FACT THAT THE FD ALWAYS INCREASES WHEN CONNECTION IS DROPPED AND RE-ESTABLISHED
       EVENTUALLY THIS WILL RESULT IN THE FD IN OVERFLOWING 
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

#define POLLING_TIMEOUT_MS 1000

int main(int argc, char *argv[])
{
    char *buffer = (char *)malloc(MSG_UNIT_SIZE); // Single buffer for reading and writing between clients

    if (buffer == NULL) {
        handle_error("Failed  to allocate buffer");
    }

    int ret = 0;                          // used for assessing returns of various fxn calls
    int ready;                            // how many fd are ready from the poll (have return event)

    int num_components = 3;
    ComponentStruct *dfgm_handler = component_factory("dfgm_handler", DFGM);
    ComponentStruct *coms_handler = component_factory("coms_handler", COMS);
    ComponentStruct *test_handler = component_factory("test_handler", TEST);

    // Array of pointers to components the message dispatcher interacts with
    ComponentStruct *components[3] = {dfgm_handler, coms_handler, test_handler};

    nfds_t nfds = (unsigned long int)num_components; // num of fds we are polling
    struct pollfd *pfds;                             // fd we are polling

    pfds = (struct pollfd *)calloc(nfds, sizeof(struct pollfd));

    for (nfds_t i = 0; i < num_components; i++)
    {
        pfds[i].fd = components[i]->conn_socket_fd;
        printf("pfds %lu : %d\n", i, pfds[i].fd);
        pfds[i].events = POLLIN;
    }

    for (;;)
    {
        ready = poll(pfds, nfds, POLLING_TIMEOUT_MS);
        if (ready == -1)
        {
            handle_error("polling failed\n");
        }
        // Loop over fds we are polling, check return event setting
        for (int i = 0; i < nfds; i++)
        {
            if (pfds[i].revents != 0 && pfds[i].revents & POLLIN)
            {
                // IF we are waiting for a client to send a connection request
                if (components[i]->connected == 0)
                {
                    //  Accept this conn request and get the data socket fd (returned from accept())
                    printf("WE GOT A CONNECTION \n");
                    accept_incoming_client_conn_request(components[i], &pfds[i]);
                }
                // IF we are waiting for incoming data from a connected client
                else
                {
                    int bytes_read = read_data_socket(components[i], &pfds[i], buffer);
                    if (bytes_read == 0)
                    {
                        continue;
                    }

                    if (!strncmp(buffer, "DOWN", sizeof(buffer)))
                    {
                        printf("Received DOWN - server shutting down \n");
                        goto CleanEnd;
                    }

                    int dest_id = get_msg_dest_id(buffer);

                    // Now use the msg destination ID to determine what component (socket) to send the message to
                    // loop over components array of pointers - whichever component id enum matches the read dest id is what we are writing to
                    int dest_comp_fd = get_fd_from_id(components, num_components, dest_id);
                    printf("Dest Comp ID is %d\n", dest_id);
                    if (dest_comp_fd > -1)
                {
                    printf("Sending\n");
                    for (int i = 0; i < bytes_read; i++)
                    {
                        printf(" %02x |", buffer[i]);
                    }
                    printf("\n");

                    // Ensure all bytes are written
                    int bytes_written = 0;
                    while (bytes_written < bytes_read)
                    {
                        ret = write(dest_comp_fd, buffer + bytes_written, bytes_read - bytes_written);
                        if (ret < 0)
                        {
                            perror("Write failed");
                            break;
                        }
                        bytes_written += ret;
                    }
                    printf("Bytes written: %d\n", bytes_written);
                }
                    memset(buffer, 0, MSG_UNIT_SIZE); // clear read buffer after handling data
                }
            }
        }
    }

CleanEnd:

    free(buffer);
    free(pfds);
    for (int i = 0; i < num_components; i++)
    {
        free(components[i]);
    }

    exit(EXIT_SUCCESS);
}