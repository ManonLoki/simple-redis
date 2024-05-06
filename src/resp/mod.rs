/*
思路，创建一个RedisFrame的枚举，用来保存所有的数据类型
创建一个Encoding和Decoding的trait，用来表示如何处理数据
为每一个类型 实现encoding和decoding的trait
*/

mod array;
mod boolean;
mod bulk_string;
mod double;
mod frame;
mod intenger;
mod map;
mod null;
mod set;
mod simpe_string;
mod simple_error;

use bytes::{Buf, BytesMut};
use thiserror::Error;

pub use {
    self::array::RespArray, bulk_string::BulkString, double::RespDouble, frame::RespFrame,
    map::RespMap, null::RespNull, set::RespSet, simpe_string::SimpleString,
    simple_error::SimpleError,
};

/// 预分配的缓冲区大小
const RESP_ARRAY_CAP: usize = 4096;
/// 结束符
const CLRF: &[u8] = b"\r\n";
/// 结束符号长度
const CRLF_LEN: usize = CLRF.len();

/// RESP编码
#[enum_dispatch::enum_dispatch]
pub trait RespEncode {
    /// 将自己编码为Vec<u8>
    fn encode(self) -> Vec<u8>;
}
/// RESP解码
/// 如果想返回Self 那么必须是Sized
#[enum_dispatch::enum_dispatch]
pub trait RespDecode: Sized {
    /// 前缀
    const PREFIX: &'static str;
    /// 将BytesMut解码为自己
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError>;

    /// 期望的长度
    fn expect_length(buf: &[u8]) -> Result<usize, RespError>;
}

/// Resp解码异常
#[derive(Error, Debug, PartialEq, Eq)]
pub enum RespError {
    #[error("Invalid Frame: {0}")]
    InvalidFrame(String),
    #[error("Invalid Frame Type: {0}")]
    InvalidFrameType(String),
    #[error("Invalid Frame Length: {0}")]
    InvalidFrameLength(usize),
    #[error("Frame Not Complete")]
    NotComplete,
    #[error("Invalid Parse: {0}")]
    InvalidIntParse(#[from] std::num::ParseIntError),
    #[error("Invalid Parse: {0}")]
    InvalidFloatParse(#[from] std::num::ParseFloatError),
}

/// 查找结束符号
pub(crate) fn find_crlf(buf: &[u8], nth: usize) -> Option<usize> {
    // 计数
    let mut count = 0;
    // 查找结束字符"\r\n"
    for i in 1..buf.len() - 1 {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            count += 1;
            if count == nth {
                return Some(i);
            }
        }
    }

    None
}
/// 处理固定长度的数据
pub(crate) fn extract_fixed_data(
    buf: &mut BytesMut,
    expect: &str,
    expect_type: &str,
) -> Result<(), RespError> {
    // 检查长度
    if buf.len() < expect.len() {
        return Err(RespError::NotComplete);
    }

    // 检查类型标识
    if !buf.starts_with(expect.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "except ({:}) got:{:?}",
            expect_type, buf
        )));
    }
    // 移动Buf的位置 到验证的固定组合末尾
    buf.advance(expect.len());

    Ok(())
}

/// 处理简单类型数据
pub(crate) fn extract_simple_frame_data(buf: &[u8], prefix: &str) -> Result<usize, RespError> {
    // 简单数据必定为 prefix + data + \r\n
    // 所以验证最少要大于3
    if buf.len() < 3 {
        return Err(RespError::NotComplete);
    }
    // 验证前缀

    if !buf.starts_with(prefix.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "except ({:}) got:{:?}",
            prefix, buf
        )));
    }

    // 找到第一个结束符号就返回
    let end = find_crlf(buf, 1).ok_or(RespError::NotComplete)?;
    Ok(end)
}

/// 拿到标头长度
fn parse_length(buf: &[u8], prefix: &str) -> Result<(usize, usize), RespError> {
    let end = extract_simple_frame_data(buf, prefix)?;
    let s = String::from_utf8_lossy(&buf[prefix.len()..end]);

    // 根据前缀判断是否是负数
    if s.starts_with('-') {
        if s.starts_with("-1") {
            Ok((end, 0))
        } else {
            Err(RespError::InvalidFrame(format!("Invalid Length:{}", s)))
        }
    } else {
        Ok((end, s.parse()?))
    }
}

/// 计算总长度
fn calc_total_length(buf: &[u8], end: usize, len: usize, prefix: &str) -> Result<usize, RespError> {
    // 计算总长度
    let mut total = end + CRLF_LEN;
    let mut data = &buf[total..];

    match prefix {
        "*" | "~" => {
            // 数组和集合只处理元素的长度
            for _ in 0..len {
                let len = RespFrame::expect_length(data)?;
                data = &data[len..];
                total += len;
            }
            Ok(total)
        }
        "%" => {
            // Map需要处理Key的长度和Value的长度
            for _ in 0..len {
                let len = SimpleString::expect_length(data)?;

                data = &data[len..];
                total += len;

                let len = RespFrame::expect_length(data)?;

                data = &data[len..];
                total += len;
            }
            Ok(total)
        }
        _ => Ok(len + CRLF_LEN),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_find_crlf() {
        let buf = b"hello\r\nworld\r\n";
        let pos = find_crlf(buf, 1).unwrap();
        assert_eq!(pos, 5);

        let pos = find_crlf(buf, 2).unwrap();
        assert_eq!(pos, 12);
    }
    #[test]
    fn test_extract_simple_frame_data() {
        let buf = b"+hello\r\nworld\r\n";
        let pos = extract_simple_frame_data(buf, "+").unwrap();
        assert_eq!(pos, 6);

        let buf = b"+hello\r\nworld\r\n";
        let pos = extract_simple_frame_data(buf, "-");
        assert!(pos.is_err());
    }

    #[test]
    fn test_extract_fixed_data() {
        let mut buf = BytesMut::from("+hello\r\nworld\r\n");
        let pos = extract_fixed_data(&mut buf, "+hello\r\n", "SimpleString");
        assert!(pos.is_ok());

        let mut buf = BytesMut::from("+hello\r\nworld\r\n");
        let pos = extract_fixed_data(&mut buf, "-hello\r\n", "SimpleError");
        assert!(pos.is_err());
    }
}
