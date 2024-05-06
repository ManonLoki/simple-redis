use bytes::BytesMut;

use crate::{extract_fixed_data, RespDecode, RespEncode, RespError};

/// RespNull
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct RespNull;

/// - null:"_\r\n"
impl RespEncode for RespNull {
    fn encode(self) -> Vec<u8> {
        "_\r\n".to_string().into_bytes()
    }
}

/// - null:"_\r\n"
impl RespDecode for RespNull {
    const PREFIX: &'static str = "_";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, "_\r\n", Self::PREFIX)?;
        Ok(RespNull)
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(3)
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;
    use anyhow::Result;
    use bytes::BytesMut;

    use super::*;
    #[test]
    fn test_encode_null() {
        let frame: RespFrame = RespNull.into();
        let result = frame.encode();
        assert_eq!(result, b"_\r\n");
    }
    #[test]
    fn test_decode_null() -> Result<()> {
        // 正常逻辑
        let mut buf = BytesMut::from("_\r\n");
        let frame = RespNull::decode(&mut buf)?;
        assert_eq!(frame, RespNull);

        Ok(())
    }
}
