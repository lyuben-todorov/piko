#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut};
    use crate::proto::{Header, Type};
    use std::convert::{TryFrom};

    #[test]
    fn test_decode_header() {
        let mut header = Header { id: 65535, size: 0, meta: 0, parcel_type: Type::DscReq };

        let mut encoded_header = bincode::serialize(&header).unwrap();
        //decode
        let serialized_header: Header = bincode::deserialize(&encoded_header.to_vec()).unwrap();


        assert_eq!(serialized_header.id, 65535);
        assert_eq!(serialized_header.size, 0);
        assert_eq!(serialized_header.meta, 0);
        assert_eq!(serialized_header.parcel_type, Type::DscReq)
    }


    #[test]
    fn test_decode_node() {
        let header = Header {
            id: 0,
            size: 8,
            meta: 0,
            parcel_type: Type::DscReq,
        };

        let bytes: Vec<u8> = bincode::serialize(&header).unwrap();

        let deserialized_header = bincode:: deserialize(bytes.as_slice()).unwrap();
        assert_eq!(header,deserialized_header)
    }
}