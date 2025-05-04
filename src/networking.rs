use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

// Handle an incoming client connection (server role)
pub fn handle_incoming_client(mut stream: TcpStream, tx: Sender<String>) {
    let peer_addr = stream.peer_addr().unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut buffer = String::new();

    loop {
        buffer.clear();
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                println!("Connection closed by peer: {}", peer_addr);
                break;
            }
            Ok(_) => {
                let message = buffer.trim();
                if !message.is_empty() {
                    let formatted = format!("From {}: {}", peer_addr, message);
                    tx.send(formatted).expect("Failed to send message to main thread");
                }
            }
            Err(e) => {
                eprintln!("Error reading from {}: {}", peer_addr, e);
                break;
            }
        }
    }
}

// Handle the outgoing connection (client role)
pub fn handle_outgoing_client(mut stream: TcpStream, tx: Sender<String>) {
    let peer_addr = stream.peer_addr().unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut buffer = String::new();

    loop {
        buffer.clear();
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                println!("Connection closed by peer: {}", peer_addr);
                break;
            }
            Ok(_) => {
                let message = buffer.trim();
                if !message.is_empty() {
                    let formatted = format!("From {}: {}", peer_addr, message);
                    tx.send(formatted).expect("Failed to send message to main thread");
                }
            }
            Err(e) => {
                eprintln!("Error reading from {}: {}", peer_addr, e);
                break;
            }
        }
    }
}

// Run the server to listen for incoming connections
pub fn run_server(addr: String, tx: Sender<String>) {
    let listener = TcpListener::bind(&addr).expect("Failed to bind to address");
    println!("Listening on {}", listener.local_addr().unwrap());

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let tx = tx.clone();
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move || {
                    handle_incoming_client(stream, tx);
                });
            }
            Err(e) => eprintln!("Error accepting connection: {}", e),
        }
    }
}

// Connect to another peer (client role)
pub fn connect_to_peer(peer_addr: String, tx: Sender<String>) {
    loop {
        match TcpStream::connect(&peer_addr) {
            Ok(stream) => {
                println!("Connected to peer: {}", peer_addr);
                let tx = tx.clone();
                thread::spawn(move || {
                    handle_outgoing_client(stream, tx);
                });
                break;
            }
            Err(e) => {
                eprintln!("Failed to connect to {}: {}. Retrying in 2 seconds...", peer_addr, e);
                thread::sleep(Duration::from_secs(2));
            }
        }
    }
}

// Send a message to all connected peers
pub fn broadcast_message(message: &str, streams: &mut Vec<TcpStream>) {
    let message = format!("{}\n", message);
    streams.retain_mut(|stream| {
        match stream.write_all(message.as_bytes()) {
            Ok(()) => {
                stream.flush().is_ok()
            }
            Err(e) => {
                eprintln!("Failed to send to {}: {}", stream.peer_addr().unwrap(), e);
                false
            }
        }
    });
}