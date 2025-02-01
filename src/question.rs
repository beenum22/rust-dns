use bytes::{Bytes, BytesMut};

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub(crate) struct Question {
    qname: Vec<LabelSequence>,
    qtype: QuestionType,
    qclass: QuestionClass,
}

impl Question {
    pub(crate) fn new(qname: String, qtype: u16, qclass: u16) -> Self {
        let mut labels = Vec::new();
        for label in qname.split('.') {
            labels.push(LabelSequence {
                content: label.to_string(),
                length: label.len() as u8,
            });
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

        for label in value.qname {
            bytes.extend_from_slice(&[label.length]);
            bytes.extend_from_slice(label.content.as_bytes());
        }
        bytes.extend_from_slice(&[0]);
        bytes.extend_from_slice(&Bytes::from(value.qtype));
        bytes.extend_from_slice(&Bytes::from(value.qclass));
        bytes.freeze()
    }
}

impl From<Bytes> for Question {
    fn from(value: Bytes) -> Self {
        let mut index = 0;
        let mut labels = Vec::new();

        while value[index] != b'\0' {
            let mut content = String::new();
            let length = value[index] as usize;
            index += 1;
            content.push_str(std::str::from_utf8(&value[index..index + length]).unwrap()); // TODO: Handle errors here
                                                                                           // content.push_str(".");
            labels.push(LabelSequence {
                content,
                length: length as u8,
            });
            index = length + index;
        }
        index += 1;

        if value.len() > index && value[index..].len() != 4 {
            panic!("Invalid Question length");
        };
        let qtype = QuestionType::from(u16::from_be_bytes([value[index], value[index + 1]]));
        let qclass = QuestionClass::from(u16::from_be_bytes([value[index + 2], value[index + 3]]));
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
                LabelSequence {
                    content: "codecrafters".to_string(),
                    length: 12,
                },
                LabelSequence {
                    content: "io".to_string(),
                    length: 2,
                },
            ]
        );
        assert_eq!(question.qtype, QuestionType::A);
        assert_eq!(question.qclass, QuestionClass::IN);
    }

    #[test]
    fn test_question_to_bytes() {
        let bytes_sample: [u8; 18] = [
            3, 119, 119, 119, 4, 116, 101, 115, 116, 3, 99, 111, 109, 0, 0, 1, 0, 1,
        ];

        let question = Question {
            qname: vec![
                LabelSequence {
                    content: "www".to_string(),
                    length: 3,
                },
                LabelSequence {
                    content: "test".to_string(),
                    length: 4,
                },
                LabelSequence {
                    content: "com".to_string(),
                    length: 3,
                },
            ],
            qtype: QuestionType::A,
            qclass: QuestionClass::IN,
        };
        assert_eq!(Bytes::from(question).as_ref(), bytes_sample);
    }

    #[test]
    fn test_question_from_bytes() {
        let bytes_sample: [u8; 18] = [
            3, 119, 119, 119, 4, 116, 101, 115, 116, 3, 99, 111, 109, 0, 0, 1, 0, 1,
        ];

        let question = Question {
            qname: vec![
                LabelSequence {
                    content: "www".to_string(),
                    length: 3,
                },
                LabelSequence {
                    content: "test".to_string(),
                    length: 4,
                },
                LabelSequence {
                    content: "com".to_string(),
                    length: 3,
                },
            ],
            qtype: QuestionType::A,
            qclass: QuestionClass::IN,
        };
        assert_eq!(
            Question::from(Bytes::copy_from_slice(&bytes_sample)),
            question
        );
    }
}
