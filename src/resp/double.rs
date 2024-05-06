use std::hash::Hash;

use bytes::BytesMut;

use crate::{extract_simple_frame_data, RespDecode, RespEncode, RespError};

use super::CRLF_LEN;

// 使用RespDouble 包装F64 以实现Eq
#[derive(Debug, Clone, PartialOrd)]
pub struct RespDouble(f64);

impl PartialEq for RespDouble {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0).abs() < f64::EPSILON
    }
}
impl Eq for RespDouble {}

impl Hash for RespDouble {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

// - double:",[+>]<integral>[.<fractional>][<Ee>[sign]<exponent>]\r\n"
impl RespEncode for RespDouble {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(32);

        let ret = if self.0.abs() > 1e+8 || self.0.abs() < 1e-8 {
            format!(",{:+e}\r\n", self.0)
        } else {
            let sign = if self.0 < 0.0 { "" } else { "+" };
            format!(",{}{:}\r\n", sign, self.0)
        };

        buf.extend_from_slice(&ret.into_bytes());
        buf
    }
}

/// - double:",[+>]<integral>[.<fractional>][<Ee>[sign]<exponent>]\r\n"
impl RespDecode for RespDouble {
    const PREFIX: &'static str = ",";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        // 获取第一个CRLF
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;

        let data = buf.split_to(end + CRLF_LEN);

        let s = String::from_utf8_lossy(&data[Self::PREFIX.len()..end]);
        // 转换为i64
        let num = s.parse::<f64>()?.into();
        Ok(num)
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

/// 下面这俩转换貌似不行

/// 实现From f64
impl From<f64> for RespDouble {
    fn from(f: f64) -> Self {
        RespDouble(f)
    }
}
/// 同时实现From f32
impl From<f32> for RespDouble {
    fn from(f: f32) -> Self {
        RespDouble(f as f64)
    }
}

impl RespDouble {
    pub fn new(f: f64) -> Self {
        RespDouble(f)
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;
    use anyhow::Result;
    use bytes::BytesMut;

    use super::*;
    #[test]
    fn test_encode_double() {
        let frame: RespFrame = RespDouble::new(1.25).into();
        let result = frame.encode();
        assert_eq!(result, b",+1.25\r\n");

        let frame: RespFrame = RespDouble::new(-1.25).into();
        let result = frame.encode();
        assert_eq!(result, b",-1.25\r\n");

        let frame: RespFrame = RespDouble::new(1.0e+10).into();
        let result = frame.encode();
        assert_eq!(result, b",+1e10\r\n");

        let frame: RespFrame = RespDouble::new(1.0e-10).into();
        let result = frame.encode();
        assert_eq!(result, b",+1e-10\r\n");
    }

    #[test]
    fn test_decode_double() -> Result<()> {
        // 正常逻辑
        let mut buf = BytesMut::from(",12.34\r\n");
        let frame = RespDouble::decode(&mut buf)?;
        assert_eq!(frame, RespDouble::new(12.34));
        // 正常逻辑2
        buf.extend_from_slice(b",-12.34\r\n");
        let frame = RespDouble::decode(&mut buf)?;
        assert_eq!(frame, RespDouble::new(-12.34));

        // 正常逻辑3
        buf.extend_from_slice(b",2.5e-5\r\n");
        let frame = RespDouble::decode(&mut buf)?;
        assert_eq!(frame, RespDouble::new(2.5e-5));

        // 正常逻辑4
        buf.extend_from_slice(b",-2.5e+8\r\n");
        let frame = RespDouble::decode(&mut buf)?;
        assert_eq!(frame, RespDouble::new(-2.5e+8));

        Ok(())
    }
}
