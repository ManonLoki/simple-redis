/*
-如何解析Frame
    - simple string:"+OK\r\n"
    - error:"-Error message\r\n"
    - integer:":[<+|->]<value>\r\n"
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
mod decode;
mod encode;

use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use bytes::BytesMut;
use thiserror::Error;

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

#[enum_dispatch::enum_dispatch(RespEncode)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
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
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct SimpleString(pub(crate) String);

/// SimpleError
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct SimpleError(pub(crate) String);

/// BulkString
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct BulkString(pub(crate) Vec<u8>);

/// RespNullBulkString
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespNullBulkString;
/// RespNull
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespNull;
/// RespNullArray
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespNullArray;

/// RespArray
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespArray(pub(crate) Vec<RespFrame>);
/// RespMap
#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct RespMap(pub(crate) BTreeMap<String, RespFrame>);
/// RespSet
#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct RespSet(pub(crate) Vec<RespFrame>);

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

impl Deref for RespArray {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
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

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}

impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}

impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(s.into())
    }
}

impl RespSet {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespSet(s.into())
    }
}

impl From<&str> for SimpleString {
    fn from(s: &str) -> Self {
        SimpleString(s.to_string())
    }
}

impl From<&str> for RespFrame {
    fn from(s: &str) -> Self {
        SimpleString(s.to_string()).into()
    }
}

impl From<&str> for SimpleError {
    fn from(s: &str) -> Self {
        SimpleError(s.to_string())
    }
}

impl From<&str> for BulkString {
    fn from(s: &str) -> Self {
        BulkString(s.as_bytes().to_vec())
    }
}

impl From<&[u8]> for BulkString {
    fn from(s: &[u8]) -> Self {
        BulkString(s.to_vec())
    }
}

impl From<&[u8]> for RespFrame {
    fn from(s: &[u8]) -> Self {
        BulkString(s.to_vec()).into()
    }
}

impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec())
    }
}

impl<const N: usize> From<&[u8; N]> for RespFrame {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec()).into()
    }
}

impl AsRef<str> for SimpleString {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<[u8]> for BulkString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
