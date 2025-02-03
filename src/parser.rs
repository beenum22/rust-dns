use bytes::Bytes;
use tokio_util::codec::{Decoder, Encoder};

use crate::{answer::Answer, header::Header, question::{self, Question}};

#[derive(Debug, PartialEq)]
pub(crate) struct UdpPacket {
    pub(crate) header: Header,
    pub(crate) question: Vec<Question>,
    pub(crate) answer: Option<Vec<Answer>>,
}

pub(crate) struct Parser;

impl Parser {
    pub fn new() -> Self {
        Self
    }
}

impl Decoder for Parser {
    type Item = UdpPacket;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() == 0 {
            return Ok(None);
        };
        // TODO: Return None when invalid lengths
        let header = Header::from(src.split_to(12).freeze());
        let mut questions = Vec::new();
        while src.len() != 0 {
            questions.push(Question::from(src.split_to(src.len()).freeze()));
        }
        Ok(Some(UdpPacket {
            header,
            question: questions,
            answer: None,
        }))
    }
}

impl Encoder<UdpPacket> for Parser {
    type Error = std::io::Error;

    fn encode(&mut self, item: UdpPacket, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        dst.extend_from_slice(&Bytes::from(item.header));
        for q in item.question {
            dst.extend_from_slice(&Bytes::from(q));
        }
        match item.answer {
            Some(answer) => {
                for a in answer {
                    dst.extend_from_slice(&Bytes::from(a))
                }
            },
            None => (),
        }
        Ok(())
    }
}

#[cfg(test)]
mod parser_tests {
    use crate::question::LabelSequence;

    use super::*;

    #[test]
    fn test_parser() {
        let mut parser = Parser;
        let mut buf = bytes::BytesMut::new();
        buf.extend_from_slice(&[
            0x04, 0xd2, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 119, 119, 119, 4, 116, 101, 115, 116,
            3, 99, 111, 109, 0, 0, 1, 0, 1, 3, 99, 111, 109, 0, 0, 1, 0, 1,
        ]);

        let packet = parser.decode(&mut buf);
        assert!(packet.is_ok());
        assert!(packet.as_ref().unwrap().is_some());
        assert_eq!(
            packet.unwrap().unwrap(),
            UdpPacket {
                header: Header::new(1234, 0, 0, 0, 0, true, 0, false, false, false, false, 0, 0),
                question:vec![Question::new("www.test.com".to_string(), 1, 1)],
                answer: None,
            }
        )
    }
}
