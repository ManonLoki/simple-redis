mod codec;

use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;

use crate::{
    cmd::{Command, CommandExecutor},
    Backend, RespFrame,
};
use anyhow::Result;
use tokio_util::codec::Framed;

use self::codec::RedisCodec;

/// 处理输入的Resp
#[derive(Debug)]
struct RedisRequest {
    frame: RespFrame,
    backend: Backend,
}
/// 处理后的Resp
#[derive(Debug)]
struct RedisResponse {
    frame: RespFrame,
}

/// 处理客户端连接
pub async fn stream_handler(stream: TcpStream, backend: Backend) -> Result<()> {
    //1. 从stream获取RespFrame
    let mut framed = Framed::new(stream, RedisCodec);
    //2. 处理命令
    loop {
        match framed.next().await {
            Some(Ok(req)) => {
                // 创建RedisRequest
                let req = RedisRequest {
                    frame: req,
                    backend: backend.clone(),
                };
                // 处理请求 等待结果
                let resp = request_handler(req).await?;

                //3. 返回结果 RespFrame
                // 发送到stream里 ，由 RedisCodec解码
                framed.send(resp.frame).await?;
            }
            Some(Err(e)) => return Err(e),
            None => return Ok(()),
        }
    }
}

async fn request_handler(req: RedisRequest) -> Result<RedisResponse> {
    let RedisRequest { frame, backend } = req;
    // 尝试转换为命令
    let cmd = Command::try_from(frame)?;
    // 执行命令等结果
    let ret_frame = cmd.execute(&backend);
    Ok(RedisResponse { frame: ret_frame })
}
