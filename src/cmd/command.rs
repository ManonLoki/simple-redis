use crate::{RespArray, RespFrame};

use super::{
    hmap::HMGet, CommandError, Echo, Get, HGet, HGetAll, HSet, Ping, SAdd, SISMember, Set,
    Unrecognized,
};

/// 创建支持的命令
#[enum_dispatch::enum_dispatch(CommandExecutor)]
#[derive(Debug)]
pub enum Command {
    Get(Get),
    Set(Set),
    HGet(HGet),
    HSet(HSet),
    HMGet(HMGet),
    HGetAll(HGetAll),
    SAdd(SAdd),
    SISMember(SISMember),
    Ping(Ping),
    Unrecognized(Unrecognized),
    Echo(Echo),
}

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
                b"hmget" => Ok(HGetAll::try_from(value)?.into()),
                b"hgetall" => Ok(HGetAll::try_from(value)?.into()),
                b"ping" => Ok(Ping::try_from(value)?.into()),
                b"echo" => Ok(Echo::try_from(value)?.into()),
                b"sadd" => Ok(SAdd::try_from(value)?.into()),
                b"sismember" => Ok(SISMember::try_from(value)?.into()),
                _ => Ok(Unrecognized.into()),
            },
            _ => Err(CommandError::InvalidCommand(
                "Command must be a bulk string with first args".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cmd::CommandExecutor, Backend, RespDecode, RespNull};
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
