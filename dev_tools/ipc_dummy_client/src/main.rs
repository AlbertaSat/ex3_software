/*  This program will connect to a server written in C
   to allow for interprocess communication.
   It is currently acting as a TCP server but is a client to the msg_dispatcher
   using SOCK_SEQPACKET.
   Written by Rowan Rasmusson Summer 2024
*/
use nix::libc;
use nix::sys::socket::{self, AddressFamily, SockFlag, SockType, UnixAddr};
use nix::unistd::{read, write};
use std::env;
use std::ffi::CString;
use std::io::{self, BufRead, Read};
use std::net::{TcpStream, SocketAddr};
use std::path::Path;
use std::process;
use std::str::FromStr;
use std::os::fd::AsRawFd;
use std::net::TcpListener;

const SOCKET_PATH_PREPEND: &str = "/tmp/fifo_socket_";
const BUFFER_SIZE: usize = 1024;
const CLIENT_POLL_TIMEOUT_MS: i32 = 100;

fn create_socket() -> io::Result<i32> {
    let socket_fd = socket::socket(
        AddressFamily::Unix,
        SockType::SeqPacket,
        SockFlag::empty(),
        None,
    )?;
    Ok(socket_fd)
}

/// Function so that it can receive data from a TCP client
/// Want to keep the same polling protocol as before
fn handle_client(mut tcp_stream: TcpStream, data_socket_fd: i32) {
    let tcp_fd = tcp_stream.as_raw_fd();

    let mut poll_fds = [
        libc::pollfd {
            fd: tcp_fd,
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: data_socket_fd,
            events: libc::POLLIN,
            revents: 0,
        },
    ];

    loop {
        let ready = unsafe {
            libc::poll(
                poll_fds.as_mut_ptr(),
                poll_fds.len() as libc::nfds_t,
                CLIENT_POLL_TIMEOUT_MS,
            )
        };

        if ready == -1 {
            eprintln!("poll error");
            process::exit(1);
        }

        for poll_fd in &poll_fds {
            if poll_fd.revents != 0 {
                if poll_fd.revents & libc::POLLIN != 0 {
                    if poll_fd.fd == tcp_fd {
                        let mut tcp_buf = vec![0u8; BUFFER_SIZE];
                        let ret = tcp_stream.read(&mut tcp_buf).unwrap();

                        if ret == 0 {
                            println!("TCP client disconnected. Exiting...");
                            return;
                        } else {
                            write(data_socket_fd, &tcp_buf[..ret]).unwrap_or_else(|_| {
                                eprintln!("write error");
                                process::exit(1);
                            });
                        }
                    } else if poll_fd.fd == data_socket_fd {
                        let mut socket_buf = vec![0u8; BUFFER_SIZE];
                        let ret = read(data_socket_fd, &mut socket_buf).unwrap();

                        if ret == 0 {
                            println!("Connection to Unix server dropped. Exiting...");
                            process::exit(0);
                        } else {
                            println!("Received: {}", String::from_utf8_lossy(&socket_buf[..ret]));
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <client_id> <tcp_port>", args[0]);
        process::exit(1);
    }

    let client_name: String = args[1].clone();
    let tcp_port: u16 = args[2].parse().unwrap_or_else(|_| {
        eprintln!("Invalid TCP port number");
        process::exit(1);
    });

    let fifo_name = format!("{}{}", SOCKET_PATH_PREPEND, client_name);
    let socket_path = CString::new(fifo_name).unwrap();

    let addr = UnixAddr::new(Path::new(socket_path.to_str().unwrap())).unwrap_or_else(|err| {
        eprintln!("Failed to create UnixAddr: {}", err);
        process::exit(1);
    });

    let data_socket_fd = create_socket().unwrap_or_else(|err| {
        eprintln!("Failed to create socket: {}", err);
        process::exit(1);
    });

    println!("Attempting to connect to {}", socket_path.to_str().unwrap());

    socket::connect(data_socket_fd, &addr).unwrap_or_else(|err| {
        eprintln!("Failed to connect to server: {}", err);
        process::exit(1);
    });

    println!(
        "Successfully Connected to {}, with fd: {}",
        socket_path.to_str().unwrap(),
        data_socket_fd
    );

    let tcp_listener = TcpListener::bind(("127.0.0.1", tcp_port)).unwrap_or_else(|err| {
        eprintln!("Failed to bind TCP server: {}", err);
        process::exit(1);
    });

    println!("TCP server listening on port {}", tcp_port);

    for stream in tcp_listener.incoming() {
        match stream {
            Ok(tcp_stream) => {
                println!("New TCP client connected");
                handle_client(tcp_stream, data_socket_fd);
            }
            Err(err) => {
                eprintln!("TCP connection failed: {}", err);
            }
        }
    }
}