CC=gcc
CFLAGS=-Werror

msg_dispatcher: msg_dispatcher.c connection.c
	$(CC) msg_dispatcher.c connection.c -o msg_dispatcher $(CFLAGS)
	
dispatcher_dummy_client: dispatcher_dummy_client.c connection.c 
	$(CC) dispatcher_dummy_client.c connection.c -o dispatcher_dummy_client $(CFLAGS)

clean:
	rm -f msg_dispatcher dispatcher_dummy_client