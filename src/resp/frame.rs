use bytes::BytesMut;

use crate::{
    BulkString, RespArray, RespDecode, RespError, RespMap, RespNull, RespSet, SimpleError,
    SimpleString,
};

use super::double::RespDouble;

#[enum_dispatch::enum_dispatch(RespEncode)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum RespFrame {
    // - simple string:"+OK\r\n"
    SimpleString(SimpleString),
    // - error:"-Error message\r\n"
    Error(SimpleError),
    // - integer:":[<+|->]<value>\r\n"
    Integer(i64),
    // - bulk string:"$<Length>\r\n<data>\r\n"
    BulkString(BulkString),
    // - array:"*<number-of-elements>\r\n<element-1>...<element-n>"
    //    -"*2\r\n$3\r\nget\r\n$5\r\nhellolr\n"
    Array(RespArray),

    // - null:"_\r\n"
    Null(RespNull),
    // // - null array:"*-1\r\n"
    // NullArray(RespNullArray),
    // - boolean:"#<tf>\r\n"
    Boolean(bool),
    // - double:",[+>]<integral>[.<fractional>][<Ee>[sign]<exponent>]\r\n"
    Double(RespDouble),
    // - map:"%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
    Map(RespMap),
    // - set:"~<number-of-elements>\r\n<element-1>..<element-n>"
    Set(RespSet),
}

/// 为RespFrame实现解码
impl RespDecode for RespFrame {
    const PREFIX: &'static str = "";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'+') => {
                let frame = SimpleString::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'-') => {
                let frame = SimpleError::decode(buf)?;
                Ok(frame.into())
            }
            Some(b':') => {
                let frame = i64::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'$') => {
                let frame = BulkString::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'*') => {
                let frame = RespArray::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'_') => {
                let frame = RespNull::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'#') => {
                let frame = bool::decode(buf)?;
                Ok(frame.into())
            }
            Some(b',') => {
                let frame = RespDouble::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'%') => {
                let frame = RespMap::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'~') => {
                let frame = RespSet::decode(buf)?;
                Ok(frame.into())
            }
            None => Err(RespError::NotComplete),
            _ => Err(RespError::InvalidFrameType(format!(
                "expect_length: unknown frame type: {:?}",
                buf
            ))),
        }
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'*') => RespArray::expect_length(buf),
            Some(b'~') => RespSet::expect_length(buf),
            Some(b'%') => RespMap::expect_length(buf),
            Some(b'$') => BulkString::expect_length(buf),
            Some(b':') => i64::expect_length(buf),
            Some(b'+') => SimpleString::expect_length(buf),
            Some(b'-') => SimpleError::expect_length(buf),
            Some(b'#') => bool::expect_length(buf),
            Some(b',') => RespDouble::expect_length(buf),
            Some(b'_') => RespNull::expect_length(buf),
            _ => Err(RespError::NotComplete),
        }
    }
}

impl From<&str> for RespFrame {
    fn from(s: &str) -> Self {
        SimpleString(s.to_string()).into()
    }
}

impl From<&[u8]> for RespFrame {
    fn from(s: &[u8]) -> Self {
        BulkString(s.to_vec()).into()
    }
}

impl<const N: usize> From<&[u8; N]> for RespFrame {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec()).into()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use anyhow::Result;
//     use bytes::BytesMut;
// }
