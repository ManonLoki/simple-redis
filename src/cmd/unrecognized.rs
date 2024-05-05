use crate::{Backend, RespFrame};

use super::{CommandExecutor, RESP_OK};

/// 暂时无法处理的Command
#[derive(Debug)]
pub struct Unrecognized;

/// 实现CommandExecutor Trait
impl CommandExecutor for Unrecognized {
    fn execute(self, _backend: &Backend) -> RespFrame {
        RESP_OK.clone()
    }
}
