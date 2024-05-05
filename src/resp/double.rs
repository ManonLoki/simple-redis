use bytes::BytesMut;

use crate::{extract_simple_frame_data, RespDecode, RespEncode, RespError};

use super::CRLF_LEN;

// - double:",[+>]<integral>[.<fractional>][<Ee>[sign]<exponent>]\r\n"
impl RespEncode for f64 {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(32);

        let ret = if self.abs() > 1e+8 || self.abs() < 1e-8 {
            format!(",{:+e}\r\n", self)
        } else {
            let sign = if self < 0.0 { "" } else { "+" };
            format!(",{}{:}\r\n", sign, self)
        };

        buf.extend_from_slice(&ret.into_bytes());
        buf
    }
}

/// - double:",[+>]<integral>[.<fractional>][<Ee>[sign]<exponent>]\r\n"
impl RespDecode for f64 {
    const PREFIX: &'static str = ",";

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
    fn test_encode_double() {
        let frame: RespFrame = 1.25.into();
        let result = frame.encode();
        assert_eq!(result, b",+1.25\r\n");

        let frame: RespFrame = (-1.25).into();
        let result = frame.encode();
        assert_eq!(result, b",-1.25\r\n");

        let frame: RespFrame = 1.0e+10.into();
        let result = frame.encode();
        assert_eq!(result, b",+1e10\r\n");

        let frame: RespFrame = 1.0e-10.into();
        let result = frame.encode();
        assert_eq!(result, b",+1e-10\r\n");
    }

    #[test]
    fn test_double_decode() -> Result<()> {
        // 正常逻辑
        let mut buf = BytesMut::from(",12.34\r\n");
        let frame = f64::decode(&mut buf)?;
        assert_eq!(frame, 12.34);
        // 正常逻辑2
        buf.extend_from_slice(b",-12.34\r\n");
        let frame = f64::decode(&mut buf)?;
        assert_eq!(frame, -12.34);

        // 正常逻辑3
        buf.extend_from_slice(b",2.5e-5\r\n");
        let frame = f64::decode(&mut buf)?;
        assert_eq!(frame, 2.5e-5);

        // 正常逻辑4
        buf.extend_from_slice(b",-2.5e+8\r\n");
        let frame = f64::decode(&mut buf)?;
        assert_eq!(frame, -2.5e+8);

        Ok(())
    }
}
