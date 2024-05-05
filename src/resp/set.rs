use std::ops::{Deref, DerefMut};

use bytes::{Buf, BytesMut};

use crate::{RespDecode, RespEncode, RespError, RespFrame};

use super::{calc_total_length, parse_length, CRLF_LEN, RESP_ARRAY_CAP};

/// RespSet
#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct RespSet(pub(crate) Vec<RespFrame>);

/// - set:"~<number-of-elements>\r\n<element-1>..<element-n>"
impl RespEncode for RespSet {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(RESP_ARRAY_CAP);

        // 获取原始长度
        let mut len = self.len();
        // 处理每一个元素
        for item in self.0 {
            let encoded = item.encode();

            // 处理重复的字节片段
            if buf.windows(encoded.len()).any(|window| window == encoded) {
                len -= 1;
                continue;
            }

            buf.extend_from_slice(&encoded);
        }

        // 处理头部
        let mut header = format!("~{}\r\n", len).into_bytes();
        header.extend_from_slice(&buf);

        header
    }
}

///  - set:"~<number-of-elements>\r\n<element-1>..<element-n>"
impl RespDecode for RespSet {
    const PREFIX: &'static str = "~";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        // 获取长度
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;
        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + CRLF_LEN);

        let mut set = RespSet::default();
        for _ in 0..len {
            let frame = RespFrame::decode(buf)?;
            set.push(frame);
        }
        Ok(set)
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total = calc_total_length(buf, end, len, Self::PREFIX)?;
        Ok(total)
    }
}

impl RespSet {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespSet(s.into())
    }
}

impl Deref for RespSet {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for RespSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::{BulkString, RespError};
    use anyhow::Result;
    use bytes::BytesMut;

    use super::*;
    #[test]
    fn test_encode_set() {
        let mut set = RespSet::default();
        set.push(BulkString::new(b"hello").into());
        set.push(100.into());
        //去重 这里过不去
        set.push(BulkString::new(b"hello").into());

        let frame: RespFrame = set.into();

        let result = frame.encode();

        println!("{:?}", String::from_utf8(result.clone()).unwrap());

        assert_eq!(result, b"~2\r\n$5\r\nhello\r\n:+100\r\n");
    }
    #[test]
    fn test_decode_set() -> Result<()> {
        // 正常逻辑
        let mut buf = BytesMut::from("~2\r\n$3\r\nval\r\n$5\r\nhello\r\n");
        let frame = RespSet::decode(&mut buf)?;
        assert_eq!(
            frame,
            RespSet::new(vec![
                BulkString::new("val").into(),
                BulkString::new("hello").into()
            ])
        );

        // 异常逻辑1
        let mut buf = BytesMut::from("~2\r\n$3\r\nval\r\n$5\r\nhello\r");
        let frame = RespSet::decode(&mut buf);
        assert_eq!(frame.unwrap_err(), RespError::NotComplete);

        Ok(())
    }
}
