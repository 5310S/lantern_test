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
            Ok(n) => {
                let message = buffer.trim();
                if !message.is_empty() {
                    let formatted = format!("From {}: {}", peer_addr, message);
                    println!("Received from {}: {} ({} bytes)", peer_addr, message, n);
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
pub fn handle_outgoing_client(mut stream: TcpStream, tx: Sender<String>, peer_addr: String) {
    let peer_addr_display = stream.peer_addr().unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut buffer = String::new();

    loop {
        buffer.clear();
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                println!("Connection closed by peer: {}", peer_addr_display);
                // Attempt to reconnect
                match TcpStream::connect(&peer_addr) {
                    Ok(new_stream) => {
                        println!("Reconnected to peer: {}", peer_addr);
                        stream = new_stream;
                        reader = BufReader::new(stream.try_clone().unwrap());
                        continue;
                    }
                    Err(e) => {
                        eprintln!("Failed to reconnect to {}: {}. Retrying in 2 seconds...", peer_addr, e);
                        thread::sleep(Duration::from_secs(2));
                        continue;
                    }
                }
            }
            Ok(n) => {
                let message = buffer.trim();
                if !message.is_empty() {
                    let formatted = format!("From {}: {}", peer_addr_display, message);
                    println!("Received from {}: {} ({} bytes)", peer_addr_display, message, n);
                    tx.send(formatted).expect("Failed to send message to main thread");
                }
            }
            Err(e) => {
                eprintln!("Error reading from {}: {}", peer_addr_display, e);
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

// Connect to another peer (client role) and return the TcpStream
pub fn connect_to_peer(peer_addr: String, tx: Sender<String>) -> TcpStream {
    loop {
        match TcpStream::connect(&peer_addr) {
            Ok(stream) => {
                println!("Connected to peer: {}", peer_addr);
                let tx_clone = tx.clone();
                let peer_addr_clone = peer_addr.clone();
                let stream_clone = stream.try_clone().expect("Failed to clone stream");
                thread::spawn(move || {
                    handle_outgoing_client(stream_clone, tx_clone, peer_addr_clone);
                });
                return stream;
            }
            Err(e) => {
                eprintln!("Failed to connect to {}: {}. Retrying in 2 seconds...", peer_addr, e);
                thread::sleep(Duration::from_secs(2));
            }
        }
    }
}

// Send a message to a single TcpStream, reconnecting if necessary
pub fn send_message(message: &str, stream: &mut Option<TcpStream>, peer_addr: &str) -> bool {
    let message = format!("{}\n", message);
    
    // If no stream or stream is invalid, try to reconnect
    if stream.is_none() || stream.as_ref().unwrap().peer_addr().is_err() {
        match TcpStream::connect(peer_addr) {
            Ok(new_stream) => {
                println!("Reconnected to peer: {}", peer_addr);
                *stream = Some(new_stream);
            }
            Err(e) => {
                eprintln!("Failed to reconnect to {}: {}", peer_addr, e);
                return false;
            }
        }
    }

    // Send the message
    if let Some(inner_stream) = stream {
        match inner_stream.write_all(message.as_bytes()) {
            Ok(()) => {
                let success = inner_stream.flush().is_ok();
                if success {
                    println!("Sent to {}: {}", peer_addr, message.trim());
                }
                success
            }
            Err(e) => {
                eprintln!("Failed to send to {}: {}", peer_addr, e);
                *stream = None; // Invalidate stream on error
                false
            }
        }
    } else {
        false
    }
}