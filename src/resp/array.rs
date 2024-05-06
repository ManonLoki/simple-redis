use std::ops::Deref;

use bytes::{Buf, BytesMut};

use crate::{
    resp::{calc_total_length, parse_length, CRLF_LEN},
    RespDecode, RespEncode, RespError, RespFrame,
};

use super::RESP_ARRAY_CAP;

/// RespArray
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct RespArray(pub(crate) Vec<RespFrame>);

/// - array:"*<number-of-elements>\r\n<element-1>...<element-n>"
///    -"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
///
/// - null array:"*-1\r\n"
impl RespEncode for RespArray {
    fn encode(self) -> Vec<u8> {
        // 创建buf
        let mut buf = Vec::with_capacity(RESP_ARRAY_CAP);
        // 只要确定len就好了
        if self.len() == 0 {
            buf.extend_from_slice(b"*-1\r\n");
        } else {
            // 先确定length
            buf.extend_from_slice(&format!("*{}\r\n", self.len()).into_bytes());
            // 遍历自身 把每一个元素放进去
            for item in self.0 {
                buf.extend_from_slice(&item.encode());
            }

            // 最后确定结尾
            buf.extend_from_slice(b"\r\n");
        };

        buf
    }
}

/// - array:"*<number-of-elements>\r\n<element-1>...<element-n>"
///   -"*2\r\n$3\r\nget\r\n$5\r\nhellolr\n"
///
/// - null array:"*-1\r\n"
impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;

        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;

        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF_LEN);

        let mut frames = Vec::with_capacity(len);
        for _ in 0..len {
            println!("buf:{:?}", String::from_utf8_lossy(buf));
            frames.push(RespFrame::decode(buf)?);
        }

        Ok(RespArray::new(frames))
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;

        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(s.into())
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::BulkString;
    use anyhow::Result;
    use bytes::BytesMut;

    use super::*;

    #[test]
    fn test_encode_array() {
        let frame: RespFrame = RespArray::new(vec![
            BulkString::new(b"get").into(),
            BulkString::new(b"hello").into(),
        ])
        .into();
        let result = frame.encode();

        println!("result:{:?}", String::from_utf8_lossy(&result));

        assert_eq!(&result, b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n\r\n");
    }
    #[test]
    fn test_decode_array() -> Result<()> {
        // 正常逻辑
        let mut buf = BytesMut::from("*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(
            frame,
            RespArray::new(vec![
                BulkString::new("get").into(),
                BulkString::new("hello").into()
            ])
        );
        // 异常逻辑1
        let mut buf = BytesMut::from("*2\r\n$3\r\nget\r\n$5\r\nhello\r");
        let frame = RespArray::decode(&mut buf);
        assert_eq!(frame.unwrap_err(), RespError::NotComplete);

        // 异常逻辑2
        let mut buf = BytesMut::from("*5\r\n$3\r\nget\r\n$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf);
        assert_eq!(frame.unwrap_err(), RespError::NotComplete);

        Ok(())
    }

    #[test]
    fn test_encode_null_array() -> Result<()> {
        let frame: RespFrame = RespArray::new(vec![]).into();
        let result = frame.encode();

        println!("result:{:?}", String::from_utf8_lossy(&result));

        assert_eq!(result, b"*-1\r\n");

        Ok(())
    }

    #[test]
    fn test_decode_null_array() -> Result<()> {
        // 正常逻辑
        let mut buf = BytesMut::from("*-1\r\n");
        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new(vec![]));

        // 正常逻辑2 前一个buf内容应该被消费掉
        buf.extend_from_slice("*2\r\n$3\r\nget\r\n$5\r\nhello\r\n".as_bytes());
        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(
            frame,
            RespArray::new(vec![
                BulkString::new("get").into(),
                BulkString::new("hello").into()
            ])
        );

        // 异常逻辑1
        let mut buf = BytesMut::from("*-1\r");
        let frame = RespArray::decode(&mut buf);
        assert_eq!(frame.unwrap_err(), RespError::NotComplete);

        // 异常逻辑2
        let mut buf = BytesMut::from("*-2\r\n");
        let frame = RespArray::decode(&mut buf);
        assert_eq!(
            frame.unwrap_err(),
            RespError::InvalidFrame("Invalid Length:-2".to_string())
        );
        Ok(())
    }

    /// 同时还要测试期望长度
    #[test]
    fn test_expect_length() -> Result<()> {
        let buf = b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n";
        let len = RespArray::expect_length(buf)?;
        assert_eq!(len, buf.len());

        let buf = b"*-1\r\n";
        let len = RespArray::expect_length(buf)?;
        assert_eq!(len, buf.len());

        Ok(())
    }
}
