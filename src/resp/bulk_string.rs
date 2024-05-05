use std::ops::Deref;

use bytes::{Buf, BytesMut};

use crate::{RespDecode, RespEncode, RespError};

use super::{parse_length, CRLF_LEN};

/// BulkString
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct BulkString(pub(crate) Vec<u8>);

///  - bulk string:"$<Length>\r\n<data>\r\n"
///  - null bulk string:"$-1\r\n"
impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        if self.len() == 0 {
            "$-1\r\n".to_string().into_bytes()
        } else {
            let mut buf = Vec::with_capacity(self.0.len() + 16);
            buf.extend_from_slice(&format!("${}\r\n", self.len()).into_bytes());
            buf.extend_from_slice(&self);
            buf.extend_from_slice(b"\r\n");
            buf
        }
    }
}

/// - bulk string:"$<Length>\r\n<data>\r\n"
/// - null bulk string:"$-1\r\n"
impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        // 计算长度，空串长度为0
        let (end, len) = parse_length(buf, Self::PREFIX)?;

        if len == 0 {
            // 如果长度为0 直接返回空串
            buf.advance(end + CRLF_LEN);
            Ok(BulkString::new(Vec::with_capacity(0)))
        } else {
            // 否则解析后续字符串元素
            let remained = &buf[end + CRLF_LEN..];
            if remained.len() < len + CRLF_LEN {
                return Err(RespError::NotComplete);
            }

            buf.advance(end + CRLF_LEN);

            let data = buf.split_to(len + CRLF_LEN);
            Ok(BulkString::new(data[..len].to_vec()))
        }
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        // 因为要加上长度之后的\r\n & 结束的\r\n 所以需要检查最终长度是否足够
        let len = end + CRLF_LEN + len + CRLF_LEN;
        if len > buf.len() {
            return Err(RespError::NotComplete);
        }
        Ok(len)
    }
}

impl From<&str> for BulkString {
    fn from(s: &str) -> Self {
        BulkString(s.as_bytes().to_vec())
    }
}

impl From<&[u8]> for BulkString {
    fn from(s: &[u8]) -> Self {
        BulkString(s.to_vec())
    }
}

impl From<String> for BulkString {
    fn from(s: String) -> Self {
        BulkString(s.into_bytes())
    }
}

impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec())
    }
}
impl AsRef<[u8]> for BulkString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for BulkString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;
    use anyhow::Result;
    use bytes::BytesMut;

    use super::*;

    #[test]
    fn test_encode_bulk_string() {
        let frame: RespFrame = BulkString::new(b"hello").into();
        let result = frame.encode();
        assert_eq!(result, b"$5\r\nhello\r\n");
    }

    #[test]
    fn test_decode_bulk_string() -> Result<()> {
        // 正常逻辑
        let mut buf = BytesMut::from("$5\r\nhello\r\n");
        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new("hello"));

        // 异常逻辑1
        let mut buf = BytesMut::from("$5\r\nhell\r\n");
        let frame = BulkString::decode(&mut buf);
        assert_eq!(frame.unwrap_err(), RespError::NotComplete);

        // 异常逻辑2
        let mut buf = BytesMut::from("$5\r\nhell\r");
        let frame = BulkString::decode(&mut buf);
        assert_eq!(frame.unwrap_err(), RespError::NotComplete);

        Ok(())
    }

    /// 测试空串encode
    #[test]
    fn test_encode_null_bulk_string() {
        let frame: RespFrame = BulkString::new(Vec::with_capacity(0)).into();
        let result = frame.encode();
        assert_eq!(result, b"$-1\r\n");
    }

    /// 测试空串decode
    #[test]
    fn test_decode_null_bulk_string() -> Result<()> {
        // 正常逻辑
        let mut buf = BytesMut::from("$-1\r\n");
        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(Vec::with_capacity(0)));

        // 正常逻辑2 前一个应该被消费掉
        buf.extend_from_slice("$1\r\na\r\n".as_bytes());
        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new("a"));

        // 异常逻辑1
        let mut buf = BytesMut::from("$-1\r");
        let frame = BulkString::decode(&mut buf);
        assert_eq!(frame.unwrap_err(), RespError::NotComplete);

        // 异常逻辑2
        let mut buf = BytesMut::from("$-1");
        let frame = BulkString::decode(&mut buf);
        assert_eq!(frame.unwrap_err(), RespError::NotComplete);

        Ok(())
    }
}
