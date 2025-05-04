use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

// Handle an incoming client connection (server role)
fn handle_incoming_client(mut stream: TcpStream, tx: Sender<String>) {
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
fn handle_outgoing_client(mut stream: TcpStream, tx: Sender<String>) {
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
fn run_server(addr: String, tx: Sender<String>) {
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
fn connect_to_peer(peer_addr: String, tx: Sender<String>) {
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
fn broadcast_message(message: &str, streams: &mut Vec<TcpStream>) {
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

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <local_addr> <peer_addr>", args[0]);
        eprintln!("Example: {} 0.0.0.0:8080 127.0.0.1:8081", args[0]);
        std::process::exit(1);
    }

    // Clone the arguments to create owned Strings
    let local_addr = args[1].clone();
    let peer_addr = args[2].clone();

    // Channel to send received messages to the main thread
    let (tx, rx) = mpsc::channel();
    
    // List to keep track of connected streams for broadcasting
    let mut streams: Vec<TcpStream> = Vec::new();

    // Start the server in a separate thread
    let server_tx = tx.clone();
    thread::spawn(move || {
        run_server(local_addr, server_tx);
    });

    // Connect to the peer (client role)
    let client_tx = tx.clone();
    let peer_addr_for_thread = peer_addr.clone(); // Clone for the thread
    thread::spawn(move || {
        connect_to_peer(peer_addr_for_thread, client_tx);
    });

    // Main thread: handle user input and display received messages
    let stdin = std::io::stdin();
    let mut stdin_reader = BufReader::new(stdin);
    let mut input = String::new();

    loop {
        // Check for received messages
        while let Ok(message) = rx.try_recv() {
            println!("{}", message);
        }

        // Non-blocking read from stdin
        input.clear();
        if stdin_reader.read_line(&mut input).is_ok() {
            let message = input.trim();
            if !message.is_empty() {
                // Add outgoing stream to the list if connected
                if let Ok(stream) = TcpStream::connect(&peer_addr) {
                    streams.push(stream);
                }
                broadcast_message(message, &mut streams);
            }
        }

        // Small sleep to prevent tight loop
        thread::sleep(Duration::from_millis(100));
    }
}