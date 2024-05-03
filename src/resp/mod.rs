/*
-如何解析Frame
    - simple string:"+OK\r\n"
    - error:"-Error message\r\n"
    - bulk error:"!<Length>\r\n<error>\r\n"
    - integer:"[>]<value>\r\n"
    - bulk string:"$<Length>\r\n<data>\r\n"
    - null bulk string:"$-1\r\n"
    - array:"*<number-of-elements>\r\n<element-1>...<element-n>"
        -"*2\r\n$3\r\nget\r\n$5\r\nhellolr\n"
    - null array:"*-1\r\n"
    - null:"_\r\n"
    - boolean:"#<tf>\r\n"
    - double:",[+>]<integral>[.<fractional>][<Ee>[sign]<exponent>]\r\n"
    - big number:"([+]<number>\r\n"
    - map:"%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
    - set:"~<number-of-elements>\r\n<element-1>..<element-n>"
*/

/*
思路，创建一个RedisFrame的枚举，用来保存所有的数据类型
创建一个Encoding和Decoding的trait，用来表示如何处理数据
为每一个类型 实现encoding和decoding的trait
*/
mod encode;

use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

/// RESP编码
pub trait RespEncode {
    /// 将自己编码为Vec<u8>
    fn encode(self) -> Vec<u8>;
}
/// RESP解码
pub trait RespDecode {
    /// 将Vec<u8>解码为自己
    fn decode(buf: Self) -> Result<RespFrame, String>;
}

pub enum RespFrame {
    // - simple string:"+OK\r\n"
    SimpleString(SimpleString),
    // - error:"-Error message\r\n"
    Error(SimpleError),
    // - integer:"[>]<value>\r\n"
    Integer(i64),
    // - bulk string:"$<Length>\r\n<data>\r\n"
    BulkString(BulkString),
    // - array:"*<number-of-elements>\r\n<element-1>...<element-n>"
    //    -"*2\r\n$3\r\nget\r\n$5\r\nhellolr\n"
    Array(Vec<RespFrame>),
    // - null bulk string:"$-1\r\n"
    NullBlukString(RespNullBulkString),
    // - null:"_\r\n"
    Null(RespNull),
    // - null array:"*-1\r\n"
    NullArray(RespNullArray),
    // - boolean:"#<tf>\r\n"
    Boolean(bool),
    // - double:",[+>]<integral>[.<fractional>][<Ee>[sign]<exponent>]\r\n"
    Double(f64),
    // - map:"%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
    Map(RespMap),
    // - set:"~<number-of-elements>\r\n<element-1>..<element-n>"
    Set(RespSet),
}

/// SimpleString
pub struct SimpleString(String);

/// SimpleError
pub struct SimpleError(String);

/// BulkString
pub struct BulkString(Vec<u8>);
/// RespNullBulkString
pub struct RespNullBulkString;
/// RespNull
pub struct RespNull;
/// RespNullArray
pub struct RespNullArray;
/// RespMap
pub struct RespMap(HashMap<String, RespFrame>);
/// RespSet
pub struct RespSet(HashSet<RespFrame>);

impl Deref for SimpleString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SimpleError {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for BulkString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RespMap {
    type Target = HashMap<String, RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RespSet {
    type Target = HashSet<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}
