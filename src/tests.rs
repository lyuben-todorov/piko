#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut};
    use crate::proto::{Header, Type};
    use std::convert::{TryFrom};

    #[test]
    fn test_decode_header() {
        let bytes: Vec<u8> = Vec::from([0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00001000u8, 0b00000000u8, 0b00000000u8, 0b00000000u8, 0b00000001u8]);
        let mut header = Bytes::from(bytes);

        //decode
        let serialized_header: Header = Header::try_from(&header.to_vec()).unwrap();


        assert_eq!(serialized_header.id, 65535);
        assert_eq!(serialized_header.size, 8);
        assert_eq!(serialized_header.meta, 0);
        assert_eq!(serialized_header.parcel_type, Type::DSCREQ)
    }

    #[test]
    fn test_encode_header() {
        let bytes: Vec<u8> = Vec::from([0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00001000u8, 0b00000000u8, 0b00000000u8, 0b00000000u8, 0b00000001u8]);

        let id: u16 = 65535;
        let size: u16 = 8;
        let meta: u16 = 0;
        let parcel_type = Type::DSCREQ;

        let header = Header::new(id, size, meta, parcel_type);

        let mut header_encoded = Vec::try_from(header).unwrap();

        assert_eq!(bytes, header_encoded);
    }

    #[test]
    fn test_encode_node() {
        let header = Header {
            id: 0,
            size: 8,
            meta: 0,
            parcel_type: Type::DSCREQ,
        };

        let bytes: Vec<u8> = Vec::try_from(header).unwrap();

        assert_eq!(bytes.len(), 8)
    }
}