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
    char buffer[MSG_UNIT_SIZE] = {0}; // Single buffer for reading and writing between clients
    int ret = 0;                      // used for assessing returns of various fxn calls
    int ready;                        // how many fd are ready from the poll (have return event)

    int num_components = 4;
    ComponentStruct *iris_handler = component_factory("iris_handler", IRIS);
    ComponentStruct *dfgm_handler = component_factory("dfgm_handler", DFGM);
    ComponentStruct *coms_handler = component_factory("coms_handler", COMS);
    ComponentStruct *test_handler = component_factory("test_handler", TEST);

    // Array of pointers to components the message dispatcher interacts with
    ComponentStruct *components[4] = {dfgm_handler, coms_handler, iris_handler, test_handler};

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
                    if (read_data_socket(components[i], &pfds[i], buffer) == 0)
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
                    if (dest_comp_fd > -1)
                    {
                        ret = write(dest_comp_fd, buffer, sizeof(buffer));
                        if (ret < 0)
                        {
                            printf("Write failed \n");
                        }
                    }
                    memset(buffer, 0, MSG_UNIT_SIZE); // clear read buffer after handling data
                }
            }
        }
    }

CleanEnd:

    free(pfds);
    for (int i = 0; i < num_components; i++)
    {
        free(components[i]);
    }

    exit(EXIT_SUCCESS);
}