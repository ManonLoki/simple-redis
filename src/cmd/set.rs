use crate::{RespArray, RespFrame};

use super::{extract_args, CommandError, CommandExecutor};

/// SAdd 命令  sadd key member [member ...]
#[derive(Debug)]
pub struct SAdd {
    pub key: String,
    pub members: Vec<RespFrame>,
}

impl CommandExecutor for SAdd {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        backend.sadd(self.key, self.members).into()
    }
}

impl TryFrom<RespArray> for SAdd {
    type Error = crate::cmd::CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        // 验证命令
        crate::cmd::validate_command(&value, &["sadd"], 2)?;

        let mut args = extract_args(value, 1)?.into_iter();

        match args.next() {
            Some(RespFrame::BulkString(key)) => {
                let key = String::from_utf8(key.0)?;
                let members = args.collect();
                Ok(SAdd { key, members })
            }
            _ => Err(CommandError::InvalidCommand("Invalid command".to_string())),
        }
    }
}

/// SISMember 命令  sismember key member
#[derive(Debug)]
pub struct SISMember {
    pub key: String,
    pub member: RespFrame,
}

impl CommandExecutor for SISMember {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        let set = backend.set.get(&self.key);
        match set {
            Some(set) => {
                if set.contains(&self.member) {
                    RespFrame::Integer(1)
                } else {
                    RespFrame::Integer(0)
                }
            }
            None => RespFrame::Integer(0),
        }
    }
}

impl TryFrom<RespArray> for SISMember {
    type Error = crate::cmd::CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        // 验证命令
        crate::cmd::validate_command(&value, &["sismember"], 2)?;

        let mut args = extract_args(value, 1)?.into_iter();

        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(member)) => {
                let key = String::from_utf8(key.0)?;
                Ok(SISMember { key, member })
            }
            _ => Err(CommandError::InvalidCommand("Invalid command".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{backend, BulkString, RespFrame};

    #[test]
    fn test_sadd() {
        let input = RespArray::new(vec![
            BulkString::new("sadd").into(),
            BulkString::new("key").into(),
            BulkString::new("member").into(),
        ]);

        let cmd = SAdd::try_from(input).unwrap();
        assert_eq!(cmd.key, "key");
        assert_eq!(cmd.members.len(), 1);
        assert_eq!(cmd.members, vec![BulkString::new("member").into()]);

        let backend = backend::Backend::new();
        let count = cmd.execute(&backend);

        assert_eq!(count, RespFrame::Integer(1));

        let input = RespArray::new(vec![
            BulkString::new("sadd").into(),
            BulkString::new("key").into(),
            BulkString::new("member").into(),
        ]);

        let cmd = SAdd::try_from(input).unwrap();
        let count = cmd.execute(&backend);
        assert_eq!(count, RespFrame::Integer(0));
    }

    #[test]
    fn test_sismember() {
        let input = RespArray::new(vec![
            BulkString::new("sismember").into(),
            BulkString::new("key").into(),
            BulkString::new("member").into(),
        ]);

        let cmd = SISMember::try_from(input).unwrap();
        assert_eq!(cmd.key, "key");
        assert_eq!(cmd.member, BulkString::new("member").into());

        let backend = backend::Backend::new();
        let count = cmd.execute(&backend);
        assert_eq!(count, RespFrame::Integer(0));

        backend.sadd("key".to_string(), vec![BulkString::new("member").into()]);
        let input = RespArray::new(vec![
            BulkString::new("sismember").into(),
            BulkString::new("key").into(),
            BulkString::new("member").into(),
        ]);

        let cmd = SISMember::try_from(input).unwrap();
        let count = cmd.execute(&backend);
        assert_eq!(count, RespFrame::Integer(1));
    }
}
