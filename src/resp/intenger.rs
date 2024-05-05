use bytes::BytesMut;

use crate::{extract_simple_frame_data, RespDecode, RespEncode, RespError};

use super::CRLF_LEN;

/// - integer:":[<+|->]<value>\r\n"
impl RespEncode for i64 {
    fn encode(self) -> Vec<u8> {
        let sign = if self < 0 { "" } else { "+" };
        format!(":{}{}\r\n", sign, self).into_bytes()
    }
}

/// - integer:":[<+|->]<value>\r\n"
impl RespDecode for i64 {
    const PREFIX: &'static str = ":";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        // 获取第一个CRLF
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;

        let data = buf.split_to(end + CRLF_LEN);

        let s = String::from_utf8_lossy(&data[Self::PREFIX.len()..end]);
        // 转换为i64
        let num = s.parse()?;
        Ok(num)
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;
    use anyhow::Result;
    use bytes::BytesMut;

    use super::*;

    #[test]
    fn test_encode_integer() {
        let frame: RespFrame = 100i64.into();
        let result = frame.encode();
        assert_eq!(result, b":+100\r\n");

        let frame: RespFrame = (-100i64).into();
        let result = frame.encode();
        assert_eq!(result, b":-100\r\n");
    }

    #[test]
    fn test_decode_integer() -> Result<()> {
        // 正常逻辑
        let mut buf = BytesMut::from(":100\r\n");
        let frame = i64::decode(&mut buf)?;
        assert_eq!(frame, 100);
        // 正常逻辑2
        let mut buf = BytesMut::from(":-100\r\n");
        let frame = i64::decode(&mut buf)?;
        assert_eq!(frame, -100);

        // 异常逻辑1
        let mut buf = BytesMut::from(":100a\r\n");
        let frame = i64::decode(&mut buf);
        assert!(frame.is_err());

        Ok(())
    }
}
