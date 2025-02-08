use bytes::{Buf, Bytes, BytesMut};

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Header {
    pub(crate) id: u16,
    pub(crate) qdcount: u16,
    pub(crate) ancount: u16,
    pub(crate) nscount: u16,
    pub(crate) arcount: u16,
    pub(crate) qr: bool,
    pub(crate) opcode: u8,
    pub(crate) aa: bool,
    pub(crate) tc: bool,
    pub(crate) rd: bool,
    pub(crate) ra: bool,
    pub(crate) z: u8,
    pub(crate) rcode: u8,
}

impl Header {
    pub(crate) fn new(
        id: u16,
        qdcount: u16,
        ancount: u16,
        nscount: u16,
        arcount: u16,
        qr: bool,
        opcode: u8,
        aa: bool,
        tc: bool,
        rd: bool,
        ra: bool,
        z: u8,
        rcode: u8,
    ) -> Self {
        Header {
            id,
            qdcount,
            ancount,
            nscount,
            arcount,
            qr,
            opcode,
            aa,
            tc,
            rd,
            ra,
            z,
            rcode,
        }
    }
}

impl From<Header> for Bytes {
    fn from(value: Header) -> Self {
        let mut bytes = BytesMut::with_capacity(12);
        bytes.extend_from_slice(&value.id.to_be_bytes());
        bytes.extend_from_slice(
            &((value.qr as u8) << 7
                | value.opcode << 3
                | (value.aa as u8) << 2
                | (value.tc as u8) << 1
                | (value.rd as u8))
                .to_be_bytes(),
        );
        bytes
            .extend_from_slice(&((value.ra as u8) << 7 | value.z << 4 | value.rcode).to_be_bytes());
        bytes.extend_from_slice(&value.qdcount.to_be_bytes());
        bytes.extend_from_slice(&value.ancount.to_be_bytes());
        bytes.extend_from_slice(&value.nscount.to_be_bytes());
        bytes.extend_from_slice(&value.arcount.to_be_bytes());
        bytes.freeze()
    }
}

impl<B: Buf> From<&mut B> for Header {
    fn from(value: &mut B) -> Self {
        if value.remaining() < 12 {
            panic!("Invalid header length");
        }
        let id = value.get_u16();
        let flags = value.get_u8();
        let qr = (flags & 0b1000_0000) >> 7 != 0;
        let opcode = (flags & 0b0111_1000) >> 3;
        let aa = (flags & 0b0000_0100) >> 2 != 0;
        let tc = (flags & 0b0000_0010) >> 1 != 0;
        let rd = flags & 0b0000_0001 != 0;
        let more_flags = value.get_u8();
        let ra = (more_flags & 0b1000_0000) >> 7 != 0;
        let z = (more_flags & 0b0111_0000) >> 4;
        let rcode =  more_flags & 0b0000_1111;
        let qdcount = value.get_u16();
        let ancount = value.get_u16();
        let nscount = value.get_u16();
        let arcount = value.get_u16();
        Header {
            id,
            qdcount,
            ancount,
            nscount,
            arcount,
            qr,
            opcode,
            aa,
            tc,
            rd,
            ra,
            z,
            rcode,
        }
    }
}

#[cfg(test)]
mod header_tests {
    use super::*;

    #[test]
    fn test_header_to_bytes() {
        let bytes_sample: [u8; 12] = [0x04, 0xd2, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let header = Header {
            id: 1234,
            qdcount: 0,
            ancount: 0,
            nscount: 0,
            arcount: 0,
            qr: true,
            opcode: 0,
            aa: false,
            tc: false,
            rd: false,
            ra: false,
            z: 0,
            rcode: 0,
        };
        let bytes = Bytes::from(header);
        assert_eq!(bytes, Bytes::copy_from_slice(&bytes_sample));
    }

    #[test]
    fn test_header_from_bytes() {
        let bytes_sample: [u8; 12] = [0x04, 0xd2, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let header_sample = Header {
            id: 1234,
            qdcount: 0,
            ancount: 0,
            nscount: 0,
            arcount: 0,
            qr: true,
            opcode: 0,
            aa: false,
            tc: false,
            rd: false,
            ra: false,
            z: 0,
            rcode: 0,
        };
        assert_eq!(
            Header::from(&mut Bytes::copy_from_slice(&bytes_sample)),
            header_sample
        );
    }
}
