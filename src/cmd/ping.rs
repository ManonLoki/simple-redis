use crate::{RespArray, SimpleString};

use super::{validate_command, CommandExecutor};

/// Ping只需要接收到命令之后返回一个Pong即可
#[derive(Debug, PartialEq)]
pub struct Ping;

/// 响应PONG简单字符串即可
impl CommandExecutor for Ping {
    fn execute(self, _backend: &crate::backend::Backend) -> crate::RespFrame {
        SimpleString::new("PONG").into()
    }
}

/// 直接实现TryFrom<RespArray> for Ping
impl TryFrom<RespArray> for Ping {
    type Error = crate::cmd::CommandError;

    fn try_from(arr: RespArray) -> Result<Self, Self::Error> {
        // 验证命令
        validate_command(&arr, &["ping"], 0)?;

        println!("Ping command: {:?}", arr);
        Ok(Ping)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RespDecode;
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_ping() {
        let ping = Ping;
        let frame = ping.execute(&crate::backend::Backend::default());
        assert_eq!(frame, SimpleString::new("PONG").into());
    }

    #[test]
    fn test_ping_try_from() -> Result<()> {
        let mut buf = BytesMut::from("*1\r\n$4\r\nping\r\n");
        let arr = RespArray::decode(&mut buf)?;
        let ping = Ping::try_from(arr).unwrap();
        assert_eq!(ping, Ping);

        Ok(())
    }
}
