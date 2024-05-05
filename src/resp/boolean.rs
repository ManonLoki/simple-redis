use bytes::BytesMut;

use crate::{extract_fixed_data, RespDecode, RespEncode, RespError};

// - boolean:"#<tf>\r\n"
impl RespEncode for bool {
    fn encode(self) -> Vec<u8> {
        let sign = if self { "t" } else { "f" };
        format!("#{}\r\n", sign).into_bytes()
    }
}

/// - boolean:"#<tf>\r\n"
impl RespDecode for bool {
    const PREFIX: &'static str = "#";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        match extract_fixed_data(buf, "#t\r\n", Self::PREFIX) {
            Ok(_) => Ok(true),
            Err(RespError::NotComplete) => Err(RespError::NotComplete),
            Err(_) => match extract_fixed_data(buf, "#f\r\n", Self::PREFIX) {
                Ok(_) => Ok(false),
                Err(RespError::NotComplete) => Err(RespError::NotComplete),
                Err(e) => Err(e),
            },
        }
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(4)
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;
    use anyhow::Result;
    use bytes::BytesMut;

    use super::*;
    #[test]
    fn test_encode_boolean() {
        let frame: RespFrame = true.into();
        let result = frame.encode();
        assert_eq!(result, b"#t\r\n");

        let frame: RespFrame = false.into();
        let result = frame.encode();
        assert_eq!(result, b"#f\r\n");
    }
    #[test]
    fn test_decode_bool() -> Result<()> {
        // 正常逻辑
        let mut buf = BytesMut::from("#t\r\n");
        let frame = bool::decode(&mut buf)?;
        assert!(frame);
        // 正常逻辑2
        buf.extend_from_slice(b"#f\r\n");
        let frame = bool::decode(&mut buf)?;
        assert!(!frame);

        Ok(())
    }
}
