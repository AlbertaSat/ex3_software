/*  This program will connect to a server written in C
   to allow for interprocess communication.
   Written by Rowan Rasmusson Summer 2024
*/

use nix::libc;
use nix::sys::socket::{self, AddressFamily, SockFlag, SockType, UnixAddr};
use nix::unistd::{read, write};
use std::env;
use std::ffi::CString;
use std::fs::File;
use std::io::{self, BufRead};
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::process;

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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <client_id>", args[0]);
        process::exit(1);
    }

    // This is the name of the handler or thing that the client is trying to connect to (fifo is named with this in path)
    let client_name: String = args[1].clone();

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

    let stdin_fd = 0; //We assume the fd for stdin is always zero. This is the default for UNIX systems and is unlikely to change.

    let mut poll_fds = [
        libc::pollfd {
            fd: stdin_fd,
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
            // println!("poll_fd: {:?}", poll_fd);
            if poll_fd.revents != 0 {
                if poll_fd.revents & libc::POLLIN != 0 {
                    if poll_fd.fd == stdin_fd {
                        println!("reading from stdin:");
                        let mut std_in_buf = String::new();
                        io::stdin().lock().read_line(&mut std_in_buf).unwrap();
                        std_in_buf = std_in_buf.trim().to_string();

                        write(data_socket_fd, std_in_buf.as_bytes()).unwrap_or_else(|_| {
                            eprintln!("write error");
                            process::exit(1);
                        });
                    } else if poll_fd.fd == data_socket_fd {
                        let mut socket_buf = vec![0u8; BUFFER_SIZE];
                        let ret = read(data_socket_fd, &mut socket_buf).unwrap();

                        if ret == 0 {
                            println!("Connection to server dropped. Exiting...");
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
