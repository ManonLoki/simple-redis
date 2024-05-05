mod hmap;
mod map;
use lazy_static::lazy_static;

use thiserror::Error;

use crate::{backend::Backend, RespArray, RespError, RespFrame, SimpleString};

lazy_static! {
    /// RESP OK 简单字符串的全局变量，这里当做常量使用
    static ref RESP_OK: RespFrame = SimpleString::new("OK").into();
}

/// CommandExecutor Trait
#[enum_dispatch::enum_dispatch]
pub trait CommandExecutor {
    /// 执行命令
    fn execute(self, backend: &Backend) -> RespFrame;
}

///  命令解析过程中的异常
#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Invalid command {0}")]
    InvalidCommand(String),
    #[error("Invalid argument {0}")]
    InvalidArgument(String),
    #[error("{0}")]
    RespError(#[from] RespError),
    #[error("From Utf8 Error:{0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

/// 创建支持的命令
#[enum_dispatch::enum_dispatch(CommandExecutor)]
#[derive(Debug)]
pub enum Command {
    Get(Get),
    Set(Set),
    HGet(HGet),
    HSet(HSet),
    HGetAll(HGetAll),

    Unrecognized(Unrecognized),
}

/// Get Command
#[derive(Debug)]
pub struct Get {
    key: String,
}

/// Set Command
#[derive(Debug)]
pub struct Set {
    key: String,
    value: RespFrame,
}

/// HGet Command
#[derive(Debug)]
pub struct HGet {
    key: String,
    field: String,
}

/// HSet Command
#[derive(Debug)]
pub struct HSet {
    key: String,
    field: String,
    value: RespFrame,
}

/// HGetAll Command
#[derive(Debug)]
pub struct HGetAll {
    key: String,
}
/// 暂时无法处理的Command
#[derive(Debug)]
pub struct Unrecognized;

/// 实现从Command到RespFrame的转换，只要RespArray
impl TryFrom<RespFrame> for Command {
    type Error = CommandError;

    fn try_from(value: RespFrame) -> Result<Self, Self::Error> {
        match value {
            RespFrame::Array(array) => array.try_into(),
            _ => Err(CommandError::InvalidCommand(
                "Command must be an array".to_string(),
            )),
        }
    }
}

/// 尝试将RespArray 转换为对应的Command
impl TryFrom<RespArray> for Command {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        // 取RespArray的第一个元素，根据协议 RespArray中的命令部分必然是一个BulkString
        match value.first() {
            // 然后判断这个字符串是否为我们支持的命令
            Some(RespFrame::BulkString(ref cmd)) => match cmd.as_ref() {
                b"get" => Ok(Get::try_from(value)?.into()),
                b"set" => Ok(Set::try_from(value)?.into()),
                b"hget" => Ok(HGet::try_from(value)?.into()),
                b"hset" => Ok(HSet::try_from(value)?.into()),
                b"hgetall" => Ok(HGetAll::try_from(value)?.into()),
                _ => Ok(Unrecognized.into()),
            },
            _ => Err(CommandError::InvalidCommand(
                "Command must be a bulk string with first args".to_string(),
            )),
        }
    }
}

/// 实现CommandExecutor Trait
impl CommandExecutor for Unrecognized {
    fn execute(self, _backend: &Backend) -> RespFrame {
        RESP_OK.clone()
    }
}

/// 验证命令是否正确 格式为 [Command .. n   Args .. n]
fn validate_command(
    value: &RespArray,
    names: &[&'static str],
    n_args: usize,
) -> Result<(), CommandError> {
    // 判断实际的Array长度，是否等于 期望的命令的数量 + 期望的参数的数量
    if value.len() != n_args + names.len() {
        return Err(CommandError::InvalidArgument(format!(
            "{} command must have exactly {} argument",
            names.join(" "),
            n_args
        )));
    }

    // 判断是否包含命令
    for (i, names) in names.iter().enumerate() {
        // 尽量用get代替 index
        match value.get(i) {
            // 因为我们的命令都是BulkString，因此这里要先确认这个是不是BulkString
            Some(RespFrame::BulkString(ref cmd)) => {
                // 然后判断命令是否匹配我们期望的命令列表 顺序+字符串
                if cmd.as_ref().to_ascii_lowercase() != names.as_bytes() {
                    return Err(CommandError::InvalidCommand(format!(
                        "Invalid command: {}",
                        String::from_utf8_lossy(cmd.as_ref())
                    )));
                }
            }
            _ => {
                return Err(CommandError::InvalidCommand(
                    "Command must be a bulk string with first args".to_string(),
                ));
            }
        }
    }

    Ok(())
}

/// 解析出命令之后的所有args
fn extract_args(value: RespArray, start: usize) -> Result<Vec<RespFrame>, CommandError> {
    let args = value.0.into_iter().skip(start).collect();
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RespDecode, RespNull};
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_command() -> Result<()> {
        // 先Decode数据
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        // 将RespArray转换为Command
        let cmd: Command = frame.try_into()?;

        // 调用Command的execute方法
        let backend = Backend::new();
        let ret = cmd.execute(&backend);
        assert_eq!(ret, RespFrame::Null(RespNull));

        // 也可以插入数据
        backend.set("hello".to_owned(), "world".into());
        buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let cmd: Command = frame.try_into()?;
        let ret = cmd.execute(&backend);

        // 最后期望world
        assert_eq!(ret, "world".into());

        Ok(())
    }
}
