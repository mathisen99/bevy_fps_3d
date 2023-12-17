use std::net::{UdpSocket, SocketAddr};
use std::collections::HashMap;
use std::str;
use std::io;

fn main() -> io::Result<()> {
    let receive_socket = UdpSocket::bind("127.0.0.1:12345")?; // Bind the receive socket
    let send_socket = UdpSocket::bind("127.0.0.1:12346")?; // Bind the send socket
    println!("Server running on 127.0.0.1:12345");

    let mut clients: HashMap<SocketAddr, String> = HashMap::new(); // Track clients

    loop {
        let mut buf = [0; 1024]; // Buffer for incoming data
        let (amt, src) = receive_socket.recv_from(&mut buf)?;

        // Process the incoming data and update the game state
        match str::from_utf8(&buf[..amt]) {
            Ok(msg) => {
                println!("Received from {}: {}", src, msg);
                clients.insert(src, msg.to_string()); // Update client state

                // Example: Echo back the message to the sender on a different port
                send_socket.send_to(msg.as_bytes(), &src)?;
            },
            Err(_) => println!("Invalid data from {}", src),
        }

        // Additional logic (e.g., broadcasting updates to clients) goes here
    }
}
