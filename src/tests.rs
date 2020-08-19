#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut, Buf};
    use crate::proto::{Header};
    use std::convert::{TryInto, TryFrom};

    #[test]
    fn test_decode_header() {
        let bytes: Vec<u8> = Vec::from([0b11101000u8, 0b10101000u8, 0b00101111u8, 0b11011001u8, 0b01110111u8, 0b01010111u8, 0b01100001u8, 0b01010100u8]);
        let mut header = Bytes::from(bytes);

        //decode
        let serialized_header: Header = Header::try_from(&header.to_vec()).unwrap();

        let id: u32 = 2002215252;
        let size: u16 = 12249;
        let meta: u16 = 59560;

        let check_header = Header { id, size, meta };

        assert_eq!(serialized_header.id, check_header.id);
        assert_eq!(serialized_header.size, check_header.size);
        assert_eq!(serialized_header.meta, check_header.meta);
    }

    #[test]
    fn test_encode_header() {
        let bytes: Vec<u8> = Vec::from([0b11101000u8, 0b10101000u8, 0b00101111u8, 0b11011001u8, 0b01110111u8, 0b01010111u8, 0b01100001u8, 0b01010100u8]);

        let id: u32 = 2002215252;
        let size: u16 = 12249;
        let meta: u16 = 59560;

        let header = Header::new(id, size, meta);

        let mut header = Vec::try_from(header).unwrap();

        let mut encoded_header = BytesMut::from(header.as_slice());

        assert_eq!(id, encoded_header.get_u32());
        assert_eq!(size, encoded_header.get_u16());
        assert_eq!(meta, encoded_header.get_u16());
    }
}