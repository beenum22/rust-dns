mod answer;
#[allow(unused_imports)]
mod header;
mod parser;
mod question;

use std::net::Ipv4Addr;

use bytes::{Bytes, BytesMut};
use futures::{SinkExt, StreamExt};
use header::Header;
use parser::{Parser, UdpPacket};
use question::{Question, QuestionType, QuestionClass};
use answer::{RData, Answer};
use tokio::net::UdpSocket;
use tokio_util::udp::UdpFramed;

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
                    let rcode = match packet.header.opcode {
                        0 => 0,
                        _ => 4,
                    };
                    let header = Header::new(
                        packet.header.id,
                        1,
                        1,
                        0,
                        0,
                        true,
                        packet.header.opcode,
                        false,
                        false,
                        packet.header.rd,
                        false,
                        0,
                        rcode,
                    );
                    let question = Question {
                        qname: packet.question.qname.clone(),
                        qtype: QuestionType::A,
                        qclass: QuestionClass::IN,
                    };
                    let answer = Answer {
                        name: packet.question.qname.clone(),
                        typ: QuestionType::A,
                        class: QuestionClass::IN,
                        ttl: 3600,
                        length: 4,
                        data: RData::A(Ipv4Addr::new(8, 8, 8, 8)),
                    };
                    if let Err(er) = sink
                        .send((
                            UdpPacket {
                                header,
                                question,
                                answer: Some(answer),
                            },
                            source,
                        ))
                        .await
                    {
                        eprintln!("Error sending data: {}", er);
                    }
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
