mod command;
mod echo;
mod hmap;
mod map;
mod ping;
mod set;
mod unrecognized;

use lazy_static::lazy_static;

use thiserror::Error;

use crate::{backend::Backend, RespArray, RespError, RespFrame, SimpleString};

pub use self::{
    command::Command,
    echo::Echo,
    hmap::{HGet, HGetAll, HMGet, HSet},
    map::{Get, Set},
    ping::Ping,
    set::{SAdd, SISMember},
    unrecognized::Unrecognized,
};
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

/// 验证命令是否正确 格式为 [Command .. n   Args .. n]
fn validate_command(
    value: &RespArray,
    names: &[&'static str],
    min_args: usize,
) -> Result<(), CommandError> {
    // 判断实际的Array长度  如果小于 期望的命令的数量 + 期望的参数的最小数量 则出错
    // 大于无所谓 可以抛弃 不然无法支持HMGet这样的命令
    if value.len() < min_args + names.len() {
        return Err(CommandError::InvalidArgument(format!(
            "{} command must have exactly {} argument",
            names.join(" "),
            min_args
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
