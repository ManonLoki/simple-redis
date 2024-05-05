use std::ops::Deref;

use bytes::BytesMut;

use crate::{extract_simple_frame_data, RespDecode, RespEncode, RespError};

use super::CRLF_LEN;

/// SimpleError
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct SimpleError(pub(crate) String);

impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}

/// - error:"-Error message\r\n"
impl RespEncode for SimpleError {
    fn encode(self) -> Vec<u8> {
        format!("-{}\r\n", self.0).into_bytes()
    }
}

///  - error:"-Error message\r\n"
impl RespDecode for SimpleError {
    const PREFIX: &'static str = "-";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        // 获取第一个CRLF
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;

        let data = buf.split_to(end + CRLF_LEN);

        let s = String::from_utf8_lossy(&data[Self::PREFIX.len()..end]);

        Ok(SimpleError::new(s))
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

impl Deref for SimpleError {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for SimpleError {
    fn from(s: &str) -> Self {
        SimpleError(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;
    use anyhow::Result;
    use bytes::BytesMut;

    use super::*;

    #[test]
    fn test_encode_simple_error() {
        let frame: RespFrame = SimpleError::new("Error message").into();
        let result = frame.encode();
        assert_eq!(result, b"-Error message\r\n");
    }

    #[test]
    fn test_decode_simple_error() -> Result<()> {
        // 正常逻辑
        let mut buf = BytesMut::from("-Error message\r\n");
        let frame = SimpleError::decode(&mut buf)?;
        assert_eq!(frame, SimpleError::new("Error message"));

        Ok(())
    }
}
