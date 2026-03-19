use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread::{self},
    time::Duration,
};

fn write_message(stream: &mut TcpStream, buf: &[u8]) {
    if let Err(x) = stream.write(buf) {
        println!("Error while writing to stream: {}", x);
    }
}

fn read_full_message(stream: &mut TcpStream) -> Vec<u8> {
    let mut buf = [0; 10];
    let mut full_message = Vec::<u8>::new();

    while let Ok(size) = stream.read(&mut buf) {
        full_message.extend_from_slice(&buf[..size]);
    }
    full_message
}

fn setup_stream(stream: &mut TcpStream) {
    if let Err(x) = stream.set_read_timeout(Some(Duration::from_millis(5))) {
        println!("Error while setting read timeout: {}", x);
    }
}

fn handle_connection(mut stream: TcpStream) {
    thread::spawn(move || {
        println!("Handling connection");

        setup_stream(&mut stream);

        let message = read_full_message(&mut stream);

        println!("Received message: {}", String::from_utf8_lossy(&message));
        write_message(&mut stream, &message);

        println!("Closing connection");
    });
}

fn main() {
    let tcp_listener = TcpListener::bind("127.0.0.1:8080");

    if let Err(x) = tcp_listener {
        println!("Tcp Listener Error: {}", x);
        return;
    }

    let tcp_listener = tcp_listener.unwrap();

    for stream in tcp_listener.incoming() {
        if let Err(x) = stream {
            println!("Error while getting stream: {}", x);
            continue;
        }

        let stream = stream.unwrap();
        handle_connection(stream);
    }
}
