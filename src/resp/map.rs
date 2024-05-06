use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use bytes::{Buf, BytesMut};

use crate::{RespDecode, RespEncode, RespError, RespFrame, SimpleString};

use super::{calc_total_length, parse_length, CRLF_LEN, RESP_ARRAY_CAP};

/// RespMap
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash, Default)]
pub struct RespMap(pub(crate) BTreeMap<String, RespFrame>);

/// - map:"%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
impl RespDecode for RespMap {
    const PREFIX: &'static str = "%";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        // 获取长度
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;
        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + CRLF_LEN);

        let mut map = RespMap::default();
        for _ in 0..len {
            let key = SimpleString::decode(buf)?;
            let value = RespFrame::decode(buf)?;
            map.insert(key.0, value);
        }
        Ok(map)
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total = calc_total_length(buf, end, len, Self::PREFIX)?;
        Ok(total)
    }
}

/// - map:"%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
impl RespEncode for RespMap {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(RESP_ARRAY_CAP);
        // 处理头部
        buf.extend_from_slice(&format!("%{}\r\n", self.0.len()).into_bytes());

        // 这里使用了BTreeMap key value 就是有序的了
        for (key, value) in self.0 {
            buf.extend_from_slice(&SimpleString::new(key).encode());
            buf.extend_from_slice(&value.encode());
        }
        buf
    }
}

impl Deref for RespMap {
    type Target = BTreeMap<String, RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RespMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::BulkString;
    use anyhow::Result;
    use bytes::BytesMut;

    use super::*;
    #[test]
    fn test_encode_map() {
        let mut map = RespMap::default();
        map.insert("key".to_string(), BulkString::new(b"value").into());
        map.insert("b".to_string(), true.into());
        let frame: RespFrame = map.into();
        let result = frame.encode();

        assert_eq!(result, b"%2\r\n+b\r\n#t\r\n+key\r\n$5\r\nvalue\r\n");
    }

    #[test]
    fn test_decode_map() -> Result<()> {
        // 正常逻辑
        let mut buf = BytesMut::from("%2\r\n+key1\r\n$3\r\nval\r\n+key2\r\n$5\r\nhello\r\n");
        let frame = RespMap::decode(&mut buf)?;
        let mut map = RespMap::default();
        map.insert("key1".to_string(), BulkString::new("val").into());
        map.insert("key2".to_string(), BulkString::new("hello").into());
        assert_eq!(frame, map);

        // 异常逻辑1
        let mut buf = BytesMut::from("%2\r\n+key1\r\n$3\r\nval\r\n+key2\r\n$5\r\nhello\r");
        let frame = RespMap::decode(&mut buf);
        assert_eq!(frame.unwrap_err(), RespError::NotComplete);

        // 异常逻辑2
        let mut buf = BytesMut::from("%3\r\n+key1\r\n$3\r\nval\r\n+key2\r\n$5\r\nhello\r\n");
        let frame = RespMap::decode(&mut buf);
        assert_eq!(frame.unwrap_err(), RespError::NotComplete);

        Ok(())
    }
}
