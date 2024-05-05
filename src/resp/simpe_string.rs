use std::ops::Deref;

use bytes::BytesMut;

use crate::{extract_simple_frame_data, RespDecode, RespEncode, RespError};

use super::CRLF_LEN;

/// SimpleString
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct SimpleString(pub(crate) String);

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}

/// - simple string:"+OK\r\n"
impl RespEncode for SimpleString {
    fn encode(self) -> Vec<u8> {
        format!("+{}\r\n", self.0).into_bytes()
    }
}

// - simple string:"+OK\r\n"
impl RespDecode for SimpleString {
    const PREFIX: &'static str = "+";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        // 获取第一个CRLF
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;

        // 获取数据
        let data = buf.split_to(end + CRLF_LEN);

        let s = String::from_utf8_lossy(&data[Self::PREFIX.len()..end]);

        Ok(SimpleString::new(s))

        // 获取第一个\r\n的位置
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

impl From<&str> for SimpleString {
    fn from(s: &str) -> Self {
        SimpleString(s.to_string())
    }
}

impl AsRef<str> for SimpleString {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl Deref for SimpleString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;
    use anyhow::Result;
    use bytes::BytesMut;

    use super::*;

    #[test]
    fn test_decode_simple_string() -> Result<()> {
        // 正常逻辑
        let mut buf = BytesMut::from("+OK\r\n");
        let frame = SimpleString::decode(&mut buf)?;
        assert_eq!(frame, SimpleString::new("OK"));

        // // 异常逻辑
        buf.extend_from_slice(b"+world\r");
        let frame = SimpleString::decode(&mut buf);
        assert_eq!(frame.unwrap_err(), RespError::NotComplete);

        // 补充\n
        buf.extend_from_slice(b"\n");
        let frame = SimpleString::decode(&mut buf)?;
        assert_eq!(frame, SimpleString::new("world"));

        Ok(())
    }

    #[test]
    fn test_encode_simple_string() {
        let frame: RespFrame = SimpleString::new("OK").into();
        let result = frame.encode();
        assert_eq!(result, b"+OK\r\n");
    }
}
