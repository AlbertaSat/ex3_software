CC=gcc
CFLAGS=-Werror

all: client server

client: client.c connection.c connection.h
	$(CC) client.c connection.c -o client $(CFLAGS)

server: server.c connection.c connection.h
	$(CC) server.c connection.c -o server $(CFLAGS)

clean:
	rm -f client server
