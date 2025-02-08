use bytes::{Buf, Bytes, BytesMut};
use log::debug;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum QuestionType {
    A,
    AAAA,
    NS,
    CNAME,
    SRV,
    PTR,
}

impl From<QuestionType> for Bytes {
    fn from(value: QuestionType) -> Self {
        match value {
            QuestionType::A => Bytes::from_static(&[0, 1]),
            QuestionType::AAAA => Bytes::from_static(&[0, 28]),
            QuestionType::NS => Bytes::from_static(&[0, 2]),
            QuestionType::CNAME => Bytes::from_static(&[0, 5]),
            QuestionType::SRV => Bytes::from_static(&[0, 33]),
            QuestionType::PTR => Bytes::from_static(&[0, 12]),
        }
    }
}

impl From<u16> for QuestionType {
    fn from(value: u16) -> Self {
        match value {
            1 => QuestionType::A,
            28 => QuestionType::AAAA,
            2 => QuestionType::NS,
            5 => QuestionType::CNAME,
            33 => QuestionType::SRV,
            12 => QuestionType::PTR,
            _ => panic!("Invalid QuestionType"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum QuestionClass {
    IN,
    CS,
    CH,
    HS,
    // IN = 1,
    // CH = 3,
    // HS = 4,
    // ANY = 255,
}

impl From<QuestionClass> for Bytes {
    fn from(value: QuestionClass) -> Self {
        match value {
            QuestionClass::IN => Bytes::from_static(&[0, 1]),
            QuestionClass::CS => Bytes::from_static(&[0, 2]),
            QuestionClass::CH => Bytes::from_static(&[0, 3]),
            QuestionClass::HS => Bytes::from_static(&[0, 4]),
        }
    }
}

impl From<u16> for QuestionClass {
    fn from(value: u16) -> Self {
        match value {
            1 => QuestionClass::IN,
            2 => QuestionClass::CS,
            3 => QuestionClass::CH,
            4 => QuestionClass::HS,
            _ => panic!("Invalid QuestionClass"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Label {
    Pointer(LabelPointer),
    Sequence(LabelSequence),
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct LabelPointer {
    pub(crate) pointer: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct LabelSequence {
    pub(crate) content: String,
    pub(crate) length: u8,
}

impl LabelSequence {
    pub(crate) fn new(content: String) -> Self {
        LabelSequence {
            length: content.len() as u8,
            content,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Question {
    pub(crate) qname: Vec<Label>,
    pub(crate) qtype: QuestionType,
    pub(crate) qclass: QuestionClass,
}

impl Question {
    pub(crate) fn new(qname: String, qtype: u16, qclass: u16) -> Self {
        let mut labels = Vec::new();
        for label in qname.split('.') {
            labels.push(Label::Sequence(LabelSequence {
                content: label.to_string(),
                length: label.len() as u8,
            }));
        }
        Question {
            qname: labels,
            qtype: QuestionType::from(qtype),
            qclass: QuestionClass::from(qclass),
        }
    }
}

impl From<Question> for Bytes {
    fn from(value: Question) -> Self {
        let mut bytes = BytesMut::new();

        for label in &value.qname {
            match label {
                Label::Pointer(pointer) => {
                    bytes.extend_from_slice(&[0b1100_0000 | (pointer.pointer >> 8) as u8]);
                    bytes.extend_from_slice(&[pointer.pointer as u8]);
                }
                Label::Sequence(sequence) => {
                    bytes.extend_from_slice(&[sequence.length]);
                    bytes.extend_from_slice(sequence.content.as_bytes());
                }
            }
        }
        if let Label::Sequence(_) = value.qname.last().unwrap() {
            bytes.extend_from_slice(&[0])
        }
        bytes.extend_from_slice(&Bytes::from(value.qtype));
        bytes.extend_from_slice(&Bytes::from(value.qclass));
        bytes.freeze()
    }
}

impl<B: Buf> From<&mut B> for Question {
    fn from(value: &mut B) -> Self {
        let mut index = 0;
        let mut labels: Vec<Label> = Vec::new();
        // let mut labels_v2: HashMap<usize, Label> = HashMap::new();
        // debug!("Question Bytes: {:02X?}", value.chunk());
        loop {
            // let first_byte = value.get_u8();
            let first_byte = value.chunk()[0];
            match (first_byte & 0b1100_0000) >> 6 {
                0 => {
                    if first_byte == b'\0' {
                        value.advance(1);
                        break;
                    }
                    let length = value.get_u8() as usize;
                    let mut content = String::new();
                    let label_bytes = value.copy_to_bytes(length).to_vec();
                    content.push_str(String::from_utf8(label_bytes.to_vec()).unwrap().as_str()); // TODO: Handle errors here
                    labels.push(Label::Sequence(LabelSequence {
                        content,
                        length: length as u8,
                    }));
                    index = length + index;
                    // }
                }
                3 => {
                    let pointer = ((value.get_u8() & 0b0011_1111) as u16) << 8 | value.get_u8() as u16;
                    labels.push(Label::Pointer(LabelPointer { pointer }));
                    break
                }
                _ => panic!("Invalid Label"),
            }
        }
        // debug!("Question Bytes After Labels: {:02X?}", value.chunk());
        let qtype = QuestionType::from(value.get_u16());
        let qclass = QuestionClass::from(value.get_u16());
        Question {
            qname: labels,
            qtype,
            qclass,
        }
    }
}

#[cfg(test)]
mod question_class_tests {
    use super::*;

    #[test]
    fn test_from_question_class_to_bytes() {
        assert_eq!(Bytes::from(QuestionClass::IN), Bytes::from_static(&[0, 1]));
        assert_eq!(Bytes::from(QuestionClass::CS), Bytes::from_static(&[0, 2]));
        assert_eq!(Bytes::from(QuestionClass::CH), Bytes::from_static(&[0, 3]));
        assert_eq!(Bytes::from(QuestionClass::HS), Bytes::from_static(&[0, 4]));
    }

    #[test]
    fn test_from_16_to_question_class() {
        assert_eq!(QuestionClass::IN, QuestionClass::from(1));
        assert_eq!(QuestionClass::CS, QuestionClass::from(2));
        assert_eq!(QuestionClass::CH, QuestionClass::from(3));
        assert_eq!(QuestionClass::HS, QuestionClass::from(4));
    }
}

mod question_type_tests {
    use super::*;

    #[test]
    fn test_from_question_type_to_bytes() {
        assert_eq!(Bytes::from(QuestionType::A), Bytes::from_static(&[0, 1]));
        assert_eq!(
            Bytes::from(QuestionType::AAAA),
            Bytes::from_static(&[0, 28])
        );
        assert_eq!(Bytes::from(QuestionType::NS), Bytes::from_static(&[0, 2]));
        assert_eq!(
            Bytes::from(QuestionType::CNAME),
            Bytes::from_static(&[0, 5])
        );
        assert_eq!(Bytes::from(QuestionType::SRV), Bytes::from_static(&[0, 33]));
        assert_eq!(Bytes::from(QuestionType::PTR), Bytes::from_static(&[0, 12]));
    }

    #[test]
    fn test_from_u16_to_question_type() {
        assert_eq!(QuestionType::A, QuestionType::from(1));
        assert_eq!(QuestionType::AAAA, QuestionType::from(28));
        assert_eq!(QuestionType::NS, QuestionType::from(2));
        assert_eq!(QuestionType::CNAME, QuestionType::from(5));
        assert_eq!(QuestionType::SRV, QuestionType::from(33));
        assert_eq!(QuestionType::PTR, QuestionType::from(12));
    }
}

mod question_tests {
    use super::*;

    #[test]
    fn test_new() {
        let question = Question::new("codecrafters.io".to_string(), 1, 1);
        assert_eq!(
            question.qname,
            vec![
                Label::Sequence(LabelSequence {
                    content: "codecrafters".to_string(),
                    length: 12,
                }),
                Label::Sequence(LabelSequence {
                    content: "io".to_string(),
                    length: 2,
                }),
            ]
        );
        assert_eq!(question.qtype, QuestionType::A);
        assert_eq!(question.qclass, QuestionClass::IN);
    }

    #[test]
    fn test_question_to_bytes() {
        let bytes_sample: [u8; 20] = [
            3,
            119,
            119,
            119,
            4,
            116,
            101,
            115,
            116,
            3,
            99,
            111,
            109,
            0b1100_0000 | 0x12,
            0x34,
            0,
            0,
            1,
            0,
            1,
        ];

        let question = Question {
            qname: vec![
                Label::Sequence(LabelSequence {
                    content: "www".to_string(),
                    length: 3,
                }),
                Label::Sequence(LabelSequence {
                    content: "test".to_string(),
                    length: 4,
                }),
                Label::Sequence(LabelSequence {
                    content: "com".to_string(),
                    length: 3,
                }),
                Label::Pointer(LabelPointer { pointer: 0x1234 }),
            ],
            qtype: QuestionType::A,
            qclass: QuestionClass::IN,
        };
        assert_eq!(Bytes::from(question).as_ref(), bytes_sample);
    }

    #[test]
    fn test_question_from_bytes() {
        let bytes_sample: [u8; 22] = [
            3,
            119,
            119,
            119,
            0b1100_0000 | 0x12,
            0x34,
            4,
            116,
            101,
            115,
            116,
            3,
            99,
            111,
            109,
            0b1100_0000 | 0x12,
            0x34,
            0,
            0,
            1,
            0,
            1,
        ];

        let question = Question {
            qname: vec![
                Label::Sequence(LabelSequence {
                    content: "www".to_string(),
                    length: 3,
                }),
                Label::Pointer(LabelPointer { pointer: 0x1234 }),
                Label::Sequence(LabelSequence {
                    content: "test".to_string(),
                    length: 4,
                }),
                Label::Sequence(LabelSequence {
                    content: "com".to_string(),
                    length: 3,
                }),
                Label::Pointer(LabelPointer { pointer: 0x1234 }),
            ],
            qtype: QuestionType::A,
            qclass: QuestionClass::IN,
        };
        assert_eq!(
            Question::from(&mut Bytes::copy_from_slice(&bytes_sample)),
            question
        );
    }
}
