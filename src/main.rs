use std::env;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
mod networking;

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
        networking::run_server(local_addr, server_tx);
    });

    // Connect to the peer (client role)
    let client_tx = tx.clone();
    let peer_addr_for_thread = peer_addr.clone(); // Clone for the thread
    thread::spawn(move || {
        networking::connect_to_peer(peer_addr_for_thread, client_tx);
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
                networking::broadcast_message(message, &mut streams);
            }
        }

        // Small sleep to prevent tight loop
        thread::sleep(Duration::from_millis(100));
    }
}