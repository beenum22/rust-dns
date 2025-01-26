use bytes::{Bytes, BytesMut};

use crate::question::{LabelSequence, QuestionType, QuestionClass};
use std::net::Ipv4Addr;

#[derive(Debug, PartialEq)]
enum RData {
    A(Ipv4Addr),
}

impl From<RData> for Bytes {
    fn from(value: RData) -> Self {
        match value {
            RData::A(ip) => Bytes::copy_from_slice(&ip.octets()),
            _ => panic!("Unsupported RData type"),
        }
    }
}

impl From<String> for RData {
    fn from(value: String) -> Self {
        match value.parse::<Ipv4Addr>() {
            Ok(ip) => RData::A(ip),
            Err(_) => panic!("Unsupported RData type"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Answer {
    name: Vec<LabelSequence>,
    typ: QuestionType,
    class: QuestionClass,
    ttl: u32,
    length: u16,
    data: RData,
}

impl Answer {
    pub(crate) fn new(name: String, typ: u16, class: u16, ttl: u32, length: u16, data: String) -> Self {
        let mut labels = Vec::new();
        for label in name.split('.') {
            labels.push(LabelSequence {
                content: label.to_string(),
                length: label.len() as u8,
            });
        }
        Answer {
            name: labels,
            typ: QuestionType::from(typ),
            class: QuestionClass::from(class),
            ttl,
            length,
            data: RData::from(data),
        }
    }
}

impl From<Answer> for Bytes {
    fn from(value: Answer) -> Self {
        let mut bytes = BytesMut::new();
        for label in value.name {
            bytes.extend_from_slice(&[label.length]);
            bytes.extend_from_slice(label.content.as_bytes());
        }
        bytes.extend_from_slice(&[0]);
        bytes.extend_from_slice(&Bytes::from(value.typ));
        bytes.extend_from_slice(&Bytes::from(value.class));
        bytes.extend_from_slice(&value.ttl.to_be_bytes());
        bytes.extend_from_slice(&value.length.to_be_bytes());
        bytes.extend_from_slice(&Bytes::from(value.data));

        bytes.freeze()
    }
}

#[cfg(test)]
mod answer_tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_rdata_from_string() {
        assert_eq!(RData::from(String::from("127.0.0.1")), RData::A(Ipv4Addr::new(127, 0, 0, 1)));
    }
    
    #[test]
    fn test_bytes_from_rdata() {
        assert_eq!(Bytes::from(RData::A(Ipv4Addr::new(127, 0, 0, 1))), Bytes::from_static(&[127, 0, 0, 1]));
    }

    #[test]
    fn test_answer_new() {
        let answer = Answer::new("codecrafters.io".to_string(), 1, 1, 3600, 4, "127.0.0.1".to_string());
        assert_eq!(answer, Answer {
            name: vec![
                LabelSequence {
                    content: "codecrafters".to_string(),
                    length: 12,
                },
                LabelSequence {
                    content: "io".to_string(),
                    length: 2,
                },
            ],
            typ: QuestionType::A,
            class: QuestionClass::IN,
            ttl: 3600,
            length: 4,
            data: RData::A(Ipv4Addr::new(127, 0, 0, 1)),
        });
    }

    #[test]
    fn test_answer_to_bytes() {
        let bytes_sample: [u8; 28] = [
            3, 119, 119, 119, 4, 116, 101, 115, 116, 3, 99, 111, 109, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 4, 127, 0, 0, 1
        ];
        let answer = Answer::new("www.test.com".to_string(), 1, 1, 0, 4, "127.0.0.1".to_string());
        let bytes = Bytes::from(answer);
        assert_eq!(bytes, Bytes::copy_from_slice(&bytes_sample));
    }
}
