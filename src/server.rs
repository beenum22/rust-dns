use std::net::{Ipv4Addr, SocketAddr, ToSocketAddrs};
use log::{debug, error, info};
use tokio::net::UdpSocket;
use crate::parser::{Parser, UdpPacket};
use crate::header::Header;
use crate::question::{QuestionClass, QuestionType};
use crate::answer::{Answer, RData};
use tokio_util::udp::UdpFramed;
use futures::{SinkExt, StreamExt};

pub(crate) struct DnsServer {
    socket: SocketAddr,
}

impl DnsServer {
    pub(crate) fn new(
        addr: String,
        port: u16,
        resolver: Option<String>,
    ) -> Self {
        Self {
            socket: format!("{addr}:{port}")
            .to_socket_addrs()
            .expect("Invalid socket address")
            .next()
            .unwrap(),
        }
    }

    pub(crate) async fn run(&self) {
        let udp_socket = match UdpSocket::bind("127.0.0.1:2053").await {
            Ok(listener) => listener,
            Err(e) => {
                error!("Failed to bind UDP listener: {}", e);
                return;
            }
        };

        info!(
            "DNS Server is running on {}:{}",
            self.socket.ip(),
            self.socket.port(),
        );

        let framed = UdpFramed::new(udp_socket, Parser::new());
        let (mut sink, mut stream) = framed.split();


        loop {
            // match udp_socket.recv_from(&mut buf).await {
            match stream.next().await {
                Some(val) => match val {
                    Ok((packet, source)) => {
                        debug!("Received {:?} packet from {}", packet, source);
                        let rcode = match packet.header.opcode {
                            0 => 0,
                            _ => 4,
                        };
                        let header = Header::new(
                            packet.header.id,
                            packet.header.qdcount,
                            packet.header.qdcount,
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
                        let mut answers = Vec::new();
                        for q in &packet.question {
                            answers.push(Answer {
                                name: q.qname.clone(),
                                typ: QuestionType::A,
                                class: QuestionClass::IN,
                                ttl: 3600,
                                length: 4,
                                data: RData::A(Ipv4Addr::new(8, 8, 8, 8)),
                            });
                        }
                        let response = UdpPacket {
                            header,
                            question: packet.question,
                            answer: Some(answers),
                        };
                        debug!("Responding with {:?} packet to {}", response, source);
                        if let Err(er) = sink.send((response, source)).await {
                            error!("Error sending data: {}", er);
                        }
                    }
                    Err(e) => {
                        error!("Error receiving data: {}", e);
                        break;
                    }
                },
                None => break,
            }
        }
    }
}
