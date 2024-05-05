use crate::{RespArray, SimpleString};

use super::{validate_command, CommandExecutor};

#[derive(Debug)]
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
