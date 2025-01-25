use bytes::{Bytes, BytesMut};

pub(crate) struct Header {
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
}

impl Header {
    pub(crate) fn new(id: u16, qdcount: u16, ancount: u16, nscount: u16, arcount: u16, qr: bool, opcode: u8, aa: bool, tc: bool, rd: bool, ra: bool, z: u8, rcode: u8) -> Self {
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
            &((value.qr as u8) << 7 | value.opcode << 3 | (value.aa as u8) << 2 | (value.tc as u8) << 1 | (value.rd as u8) ).to_be_bytes(),
        );
        bytes.extend_from_slice(
            &((value.ra as u8) << 7 | value.z << 4 | value.rcode).to_be_bytes(),
        );
        bytes.extend_from_slice(&value.qdcount.to_be_bytes());
        bytes.extend_from_slice(&value.ancount.to_be_bytes());
        bytes.extend_from_slice(&value.nscount.to_be_bytes());
        bytes.extend_from_slice(&value.arcount.to_be_bytes());
        bytes.freeze()
    }
}

#[cfg(test)]
mod tests {
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
}