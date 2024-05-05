use crate::{backend::Backend, RespArray, RespFrame};

use super::{extract_args, validate_command, CommandError, CommandExecutor, Get, Set, RESP_OK};

/// 为Get实现Executor 实际上就是去Backend中获取数据
impl CommandExecutor for Get {
    fn execute(self, backend: &Backend) -> RespFrame {
        match backend.get(&self.key) {
            Some(v) => v,
            None => RespFrame::Null(crate::RespNull),
        }
    }
}
/// 为Set实现Executor 实际上就是去Backend中设置数据
impl CommandExecutor for Set {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend.set(self.key.clone(), self.value.clone());
        RESP_OK.clone()
    }
}

/// 从RespArray中解析Get命令
impl TryFrom<RespArray> for Get {
    type Error = CommandError;

    fn try_from(arr: RespArray) -> Result<Self, Self::Error> {
        // 验证命令长度
        validate_command(&arr, &["get"], 1)?;
        // 解析参数
        let mut args = extract_args(arr, 1)?.into_iter();

        // 如果成功解析就构建Get
        match args.next() {
            Some(RespFrame::BulkString(key)) => Ok(Get {
                key: String::from_utf8(key.0)?,
            }),
            _ => Err(CommandError::InvalidCommand("Missing key".to_string())),
        }
    }
}

/// 从RespArray中解析Set命令
impl TryFrom<RespArray> for Set {
    type Error = CommandError;

    fn try_from(arr: RespArray) -> Result<Self, Self::Error> {
        // 验证命令
        validate_command(&arr, &["set"], 2)?;
        // 解析参数
        let mut args = extract_args(arr, 1)?.into_iter();

        // 这里需要解析出2个参数，如果不足或者不为BulkString就返回错误
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(value)) => Ok(Set {
                key: String::from_utf8(key.0)?,
                value,
            }),
            _ => Err(CommandError::InvalidCommand(
                "Missing key or value".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Backend, BulkString, RespDecode};
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_get_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let result: Get = frame.try_into()?;
        assert_eq!(result.key, "hello");

        Ok(())
    }

    #[test]
    fn test_set_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n");

        let frame = RespArray::decode(&mut buf)?;
        let result: Set = frame.try_into()?;
        assert_eq!(result.key, "hello");
        assert_eq!(result.value, BulkString::new("world").into());

        Ok(())
    }

    #[test]
    fn test_set_get_command() -> Result<()> {
        let backend = Backend::new();
        let cmd = Set {
            key: "hello".to_string(),
            value: BulkString::new("world").into(),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, RESP_OK.clone());

        let cmd = Get {
            key: "hello".to_string(),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, RespFrame::BulkString(b"world".into()));

        Ok(())
    }
}
