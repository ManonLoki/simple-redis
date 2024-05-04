use crate::{
    BulkString, RespArray, RespDecode, RespError, RespFrame, RespMap, RespNull, RespNullArray,
    RespNullBulkString, RespSet, SimpleError, SimpleString,
};
/// Decode 就是不停地在拆各种字符串，转换成对应的类型
/// 所有的协议起始符号 都是 1个字节的 Ascii 字符
/// 所有协议的结束符号 都是 \r\n
/// Array Map Set这三个，需要递归处理
use bytes::{Buf, BytesMut};

/// 结束符
const CLRF: &[u8] = b"\r\n";
/// 结束符号长度
const CRLF_LEN: usize = CLRF.len();

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
                // try null bulk string first
                match RespNullBulkString::decode(buf) {
                    Ok(frame) => Ok(frame.into()),
                    Err(RespError::NotComplete) => Err(RespError::NotComplete),
                    Err(_) => {
                        let frame = BulkString::decode(buf)?;
                        Ok(frame.into())
                    }
                }
            }
            Some(b'*') => {
                // try null array first
                match RespNullArray::decode(buf) {
                    Ok(frame) => Ok(frame.into()),
                    Err(RespError::NotComplete) => Err(RespError::NotComplete),
                    Err(_) => {
                        let frame = RespArray::decode(buf)?;
                        Ok(frame.into())
                    }
                }
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
                let frame = f64::decode(buf)?;
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
            Some(b',') => f64::expect_length(buf),
            Some(b'_') => RespNull::expect_length(buf),
            _ => Err(RespError::NotComplete),
        }
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

/// - null:"_\r\n"
impl RespDecode for RespNull {
    const PREFIX: &'static str = "_";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, "_\r\n", Self::PREFIX)?;
        Ok(RespNull)
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(3)
    }
}

/// - null array:"*-1\r\n"
impl RespDecode for RespNullArray {
    const PREFIX: &'static str = "*";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, "*-1\r\n", Self::PREFIX)?;
        Ok(RespNullArray)
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(5)
    }
}

/// - null bulk string:"$-1\r\n"
impl RespDecode for RespNullBulkString {
    const PREFIX: &'static str = "$";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, "$-1\r\n", Self::PREFIX)?;
        Ok(RespNullBulkString)
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(5)
    }
}

/// - integer:":[<+|->]<value>\r\n"
impl RespDecode for i64 {
    const PREFIX: &'static str = ":";

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

/// - boolean:"#<tf>\r\n"
impl RespDecode for bool {
    const PREFIX: &'static str = "#";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        match extract_fixed_data(buf, "#t\r\n", Self::PREFIX) {
            Ok(_) => Ok(true),
            Err(RespError::NotComplete) => Err(RespError::NotComplete),
            Err(_) => match extract_fixed_data(buf, "#f\r\n", Self::PREFIX) {
                Ok(_) => Ok(false),
                Err(RespError::NotComplete) => Err(RespError::NotComplete),
                Err(e) => Err(e),
            },
        }
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(4)
    }
}

/// - bulk string:"$<Length>\r\n<data>\r\n"
impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let remained = &buf[end + CRLF_LEN..];
        if remained.len() < len + CRLF_LEN {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF_LEN);

        let data = buf.split_to(len + CRLF_LEN);
        Ok(BulkString::new(data[..len].to_vec()))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN + len + CRLF_LEN)
    }
}

/// - array:"*<number-of-elements>\r\n<element-1>...<element-n>"
///   -"*2\r\n$3\r\nget\r\n$5\r\nhellolr\n"
impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;

        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;

        // 死在这里了
        println!("=============");

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

/// 查找结束符号
fn find_crlf(buf: &[u8], nth: usize) -> Option<usize> {
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
fn extract_fixed_data(
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
fn extract_simple_frame_data(buf: &[u8], prefix: &str) -> Result<usize, RespError> {
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
    Ok((end, s.parse()?))
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

                // 这里越界了
                if len > data.len() {
                    return Err(RespError::NotComplete);
                }

                data = &data[len..];
                total += len;
            }
            Ok(total)
        }
        "%" => {
            // find nth CRLF in the buffer. For map, we need to find 2 CRLF for each key-value pair
            for _ in 0..len {
                let len = SimpleString::expect_length(data)?;

                if len > data.len() {
                    return Err(RespError::NotComplete);
                }

                data = &data[len..];
                total += len;

                let len = RespFrame::expect_length(data)?;
                if len > data.len() {
                    return Err(RespError::NotComplete);
                }

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
    use bytes::BytesMut;

    use super::*;
    use anyhow::Result;

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
    fn test_decode_simple_error() -> Result<()> {
        // 正常逻辑
        let mut buf = BytesMut::from("-Error message\r\n");
        let frame = SimpleError::decode(&mut buf)?;
        assert_eq!(frame, SimpleError::new("Error message"));

        Ok(())
    }

    #[test]
    fn test_decode_null() -> Result<()> {
        // 正常逻辑
        let mut buf = BytesMut::from("_\r\n");
        let frame = RespNull::decode(&mut buf)?;
        assert_eq!(frame, RespNull);

        Ok(())
    }
    #[test]
    fn test_decode_null_bulk_string() -> Result<()> {
        // 正常逻辑
        let mut buf = BytesMut::from("$-1\r\n");
        let frame = RespNullBulkString::decode(&mut buf)?;
        assert_eq!(frame, RespNullBulkString);

        Ok(())
    }

    #[test]
    fn test_decode_integer() -> Result<()> {
        // 正常逻辑
        let mut buf = BytesMut::from(":100\r\n");
        let frame = i64::decode(&mut buf)?;
        assert_eq!(frame, 100);
        // 正常逻辑2
        let mut buf = BytesMut::from(":-100\r\n");
        let frame = i64::decode(&mut buf)?;
        assert_eq!(frame, -100);

        // 异常逻辑1
        let mut buf = BytesMut::from(":100a\r\n");
        let frame = i64::decode(&mut buf);
        assert!(frame.is_err());

        Ok(())
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

    #[test]
    fn test_decode_bool() -> Result<()> {
        // 正常逻辑
        let mut buf = BytesMut::from("#t\r\n");
        let frame = bool::decode(&mut buf)?;
        assert!(frame);
        // 正常逻辑2
        buf.extend_from_slice(b"#f\r\n");
        let frame = bool::decode(&mut buf)?;
        assert!(!frame);

        Ok(())
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
