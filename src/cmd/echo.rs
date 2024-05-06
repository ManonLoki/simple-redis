use crate::{RespArray, RespFrame};

use super::CommandExecutor;

/// Echo命令 接收一个Message，并将这个Message原样返回
/// 这里实际应该是个String，但是由于是回显，就没必要转换数据了
#[derive(Debug)]
pub struct Echo {
    message: RespFrame,
}

/// 实现CommandExecutor Trait
impl CommandExecutor for Echo {
    fn execute(self, _backend: &crate::backend::Backend) -> crate::RespFrame {
        // 返回自身消息
        self.message
    }
}

/// 实现从RespArray到Echo的转换
impl TryFrom<RespArray> for Echo {
    type Error = crate::cmd::CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        // 验证命令
        crate::cmd::validate_command(&value, &["echo"], 1)?;

        // 取出第一个参数
        let message = value[1].clone();

        Ok(Echo { message })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BulkString, RespDecode};
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_echo() -> Result<()> {
        let mut buf = BytesMut::from("*2\r\n$4\r\necho\r\n$5\r\nhello\r\n");
        let arr = RespArray::decode(&mut buf)?;
        let echo = Echo::try_from(arr).unwrap();
        assert_eq!(echo.message, BulkString::new("hello").into());

        Ok(())
    }
}
