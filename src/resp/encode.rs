use crate::{
    BulkString, RespArray, RespEncode, RespMap, RespNull, RespNullArray, RespNullBulkString,
    RespSet, SimpleError, SimpleString,
};

/// 预分配的缓冲区大小
const RESP_ARRAY_CAP: usize = 4096;

/// - simple string:"+OK\r\n"
impl RespEncode for SimpleString {
    fn encode(self) -> Vec<u8> {
        format!("+{}\r\n", self.0).into_bytes()
    }
}

/// - error:"-Error message\r\n"
impl RespEncode for SimpleError {
    fn encode(self) -> Vec<u8> {
        format!("-{}\r\n", self.0).into_bytes()
    }
}

/// - integer:":[<+|->]<value>\r\n"
impl RespEncode for i64 {
    fn encode(self) -> Vec<u8> {
        let sign = if self < 0 { "" } else { "+" };
        format!(":{}{}\r\n", sign, self).into_bytes()
    }
}

///  - bulk string:"$<Length>\r\n<data>\r\n"
impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.0.len() + 16);
        buf.extend_from_slice(&format!("${}\r\n", self.len()).into_bytes());
        buf.extend_from_slice(&self);
        buf.extend_from_slice(b"\r\n");
        buf
    }
}
/// - null bulk string:"$-1\r\n"
impl RespEncode for RespNullBulkString {
    fn encode(self) -> Vec<u8> {
        "$-1\r\n".to_string().into_bytes()
    }
}

/// - array:"*<number-of-elements>\r\n<element-1>...<element-n>"
///    -"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
impl RespEncode for RespArray {
    fn encode(self) -> Vec<u8> {
        // 创建buf
        let mut buf = Vec::with_capacity(RESP_ARRAY_CAP);
        // 先确定length
        buf.extend_from_slice(&format!("*{}\r\n", self.len()).into_bytes());

        // 遍历自身 把每一个元素放进去
        for item in self.0 {
            buf.extend_from_slice(&item.encode());
        }

        // 最后确定结尾
        buf.extend_from_slice(b"\r\n");

        buf
    }
}

/// - null:"_\r\n"
impl RespEncode for RespNull {
    fn encode(self) -> Vec<u8> {
        "_\r\n".to_string().into_bytes()
    }
}
/// - null array:"*-1\r\n"
impl RespEncode for RespNullArray {
    fn encode(self) -> Vec<u8> {
        "*-1\r\n".to_string().into_bytes()
    }
}
// - boolean:"#<tf>\r\n"
impl RespEncode for bool {
    fn encode(self) -> Vec<u8> {
        let sign = if self { "t" } else { "f" };
        format!("#{}\r\n", sign).into_bytes()
    }
}
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

/// 实现测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::RespFrame;
    #[test]
    fn test_encode_simple_string() {
        let frame: RespFrame = SimpleString::new("OK").into();
        let result = frame.encode();
        assert_eq!(result, b"+OK\r\n");
    }
    #[test]
    fn test_encode_simple_error() {
        let frame: RespFrame = SimpleError::new("Error message").into();
        let result = frame.encode();
        assert_eq!(result, b"-Error message\r\n");
    }

    #[test]
    fn test_encode_integer() {
        let frame: RespFrame = 100i64.into();
        let result = frame.encode();
        assert_eq!(result, b":+100\r\n");

        let frame: RespFrame = (-100i64).into();
        let result = frame.encode();
        assert_eq!(result, b":-100\r\n");
    }

    #[test]
    fn test_encode_bulk_string() {
        let frame: RespFrame = BulkString::new(b"hello").into();
        let result = frame.encode();
        assert_eq!(result, b"$5\r\nhello\r\n");
    }

    #[test]
    fn test_encode_array() {
        let frame: RespFrame = RespArray::new(vec![
            BulkString::new(b"get").into(),
            BulkString::new(b"hello").into(),
        ])
        .into();
        let result = frame.encode();
        assert_eq!(&result, b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n\r\n");
    }

    #[test]
    fn test_encode_null_bulk_string() {
        let frame: RespFrame = RespNullBulkString.into();
        let result = frame.encode();
        assert_eq!(result, b"$-1\r\n");
    }
    #[test]
    fn test_encode_null() {
        let frame: RespFrame = RespNull.into();
        let result = frame.encode();
        assert_eq!(result, b"_\r\n");
    }
    #[test]
    fn test_encode_null_array() {
        let frame: RespFrame = RespNullArray.into();
        let result = frame.encode();
        assert_eq!(result, b"*-1\r\n");
    }

    #[test]
    fn test_encode_boolean() {
        let frame: RespFrame = true.into();
        let result = frame.encode();
        assert_eq!(result, b"#t\r\n");

        let frame: RespFrame = false.into();
        let result = frame.encode();
        assert_eq!(result, b"#f\r\n");
    }

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
    fn test_encode_map() {
        let mut map = RespMap::default();
        map.insert("key".to_string(), BulkString::new(b"value").into());
        map.insert("b".to_string(), true.into());
        let frame: RespFrame = map.into();
        let result = frame.encode();

        assert_eq!(result, b"%2\r\n+b\r\n#t\r\n+key\r\n$5\r\nvalue\r\n");
    }

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
}
