#[allow(unused_imports)]
use std::net::UdpSocket;

mod header;
mod question;
mod answer;

use bytes::{Bytes, BytesMut};
use header::Header;
use question::Question;
use answer::Answer;

enum PacketType {
    Query,
    Response,
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];
    
    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                let mut response = BytesMut::new();
                let header = Header::new(1234, 1, 1, 0, 0, true, 0, false, false, false, false, 0, 0);
                let question = Question::new("codecrafters.io".to_string(), 1, 1);
                let answer = Answer::new("codecrafters.io".to_string(), 1, 1, 3600, 4, String::from("127.0.0.1"));
                response.extend_from_slice(&Bytes::from(header));
                response.extend_from_slice(&Bytes::from(question));
                response.extend_from_slice(&Bytes::from(answer));
                udp_socket
                    .send_to(&response, source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
