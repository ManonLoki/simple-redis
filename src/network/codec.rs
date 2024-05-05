use crate::{RespDecode, RespEncode, RespError, RespFrame};
use anyhow::Result;
use tokio_util::codec::{Decoder, Encoder};

/// Redis的Codec四线
#[derive(Debug)]
pub struct RedisCodec;
/// 将RespFrame Encode成bytes
impl Encoder<RespFrame> for RedisCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: RespFrame, dst: &mut bytes::BytesMut) -> Result<()> {
        let encode = item.encode();
        dst.extend_from_slice(&encode);
        Ok(())
    }
}

/// 将Bytes Decode成RespFrame
impl Decoder for RedisCodec {
    type Item = RespFrame;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>> {
        match RespFrame::decode(src) {
            Ok(frame) => Ok(Some(frame)),
            Err(RespError::NotComplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
