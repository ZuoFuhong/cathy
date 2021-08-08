use crate::proto::{Action, Package};
use crate::Buffer;
use crate::{IMError, Result};
use protobuf::ProtobufEnum;

// 消息类型字节数据长度
const TYPE_LEN: u8 = 2;
// 消息体字节数组长度
const BODY_LEN: u8 = 2;
// 消息头部字节数组长度
const HEAD_LEN: u8 = TYPE_LEN + BODY_LEN;
// 消息体最大长度
const CONTENT_MAX_LEN: usize = 4092;

/// 通信协议
/// --------------------------------------
/// | Type(2字节) | len(2字节) | body(len) |
/// --------------------------------------
pub struct Codec;

impl Codec {
    pub fn encode(p: Package) -> Result<Vec<u8>> {
        let body_len = p.content.len();
        if body_len as usize > CONTENT_MAX_LEN {
            return Err(IMError::ContentMaxLen);
        }
        let mut buffer = vec![0; HEAD_LEN as usize + body_len as usize];

        // 写大端序
        let action = p.action as u16;
        buffer[0] = (action >> 8) as u8;
        buffer[1] = action as u8;
        buffer[2] = (body_len >> 8) as u8;
        buffer[3] = body_len as u8;
        buffer[4..].copy_from_slice(p.get_content());
        return Ok(buffer);
    }

    pub fn decode(buffer: &mut Buffer) -> Result<Package> {
        let action_buf = buffer.seek(0, TYPE_LEN as usize)?;
        let len_buf = buffer.seek(TYPE_LEN as usize, BODY_LEN as usize)?;
        // 读大端序
        let action = action_buf[1] as u16 | (action_buf[0] as u16) << 8;
        let body_len = len_buf[1] as u16 | (len_buf[0] as u16) << 8;
        if body_len as usize > CONTENT_MAX_LEN {
            return Err(IMError::ContentMaxLen);
        }
        let content = buffer.read(HEAD_LEN as usize, body_len as usize)?;

        let mut package = Package::new();
        package.action = Action::from_i32(action as i32).unwrap();
        package.content = content;
        Ok(package)
    }
}
