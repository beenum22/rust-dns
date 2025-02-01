#[allow(unused_imports)]
mod header;
mod question;
mod answer;
mod parser;

use bytes::{Bytes, BytesMut};
use futures::{SinkExt, StreamExt};
use tokio_util::udp::UdpFramed;
use tokio::net::UdpSocket;
use header::Header;
use question::Question;
use answer::Answer;
use parser::{Parser, UdpPacket};

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");
    let udp_socket = match UdpSocket::bind("127.0.0.1:2053").await {
        Ok(listener) => listener,
        Err(e) => {
            panic!("Failed to bind TCP listener: {}", e);
        }
    };

    // let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let framed = UdpFramed::new(udp_socket, Parser::new());
    let (mut sink, mut stream) = framed.split();
    // let writer = FramedWrite::new(rx_writer, RespParser::new());
    // let mut buf = [0; 512];
    
    loop {
        // match udp_socket.recv_from(&mut buf).await {
        match stream.next().await {
            Some(val) => match val {
                Ok((packet, source)) => {
                    println!("Received {:?} packet from {}", packet, source);
                    let mut response = BytesMut::new();
                    let rcode = match packet.header.opcode {
                        0 => 0,
                        _ => 4,
                        
                    };
                    let header = Header::new(packet.header.id, 1, 1, 0, 0, true, packet.header.opcode, false, false, packet.header.rd, false, 0, rcode);
                    let question = Question::new("codecrafters.io".to_string(), 1, 1);
                    let answer = Answer::new("codecrafters.io".to_string(), 1, 1, 3600, 4, String::from("127.0.0.1"));
                    // response.extend_from_slice(&Bytes::from(header));
                    // response.extend_from_slice(&Bytes::from(question));
                    // response.extend_from_slice(&Bytes::from(answer));
                    if let Err(er) = sink.send((UdpPacket {header, question, answer: Some(answer)}, source)).await {
                        eprintln!("Error sending data: {}", er);
                    }
                    // sink.send_all((response.freeze(), source)).await.unwrap();
                }
                Err(e) => {
                    eprintln!("Error receiving data: {}", e);
                    break;
                }
            },
            None => break,
        }
    }
}
