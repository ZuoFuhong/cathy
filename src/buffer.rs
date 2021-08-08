use crate::error::IMError;
use crate::Result;
use std::io::Read;
use std::net::TcpStream;

pub const BUFFER_MAX_LEN: usize = 4096;

pub struct Buffer {
    buf: Vec<u8>,
    start: usize,
    end: usize,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            buf: vec![0; BUFFER_MAX_LEN],
            start: 0,
            end: 0,
        }
    }

    fn len(&self) -> usize {
        self.end - self.start
    }

    // 将有效的字节前移
    fn grow(&mut self) {
        if self.start == 0 {
            return;
        }
        let temp = Vec::from(&self.buf[self.start..self.end]);
        let valid_num = self.end - self.start;
        self.buf[..valid_num].copy_from_slice(&temp);
        self.end -= self.start;
        self.start = 0
    }

    // 从stream中读取字节，如果reader阻塞，发生阻塞
    pub fn read_from_reader(&mut self, stream: &mut TcpStream) -> Result<()> {
        self.grow();
        let n = stream.read(&mut self.buf[self.end..])?;
        if n == 0 {
            return Err(IMError::TcpStreamEOF);
        }
        self.end += n;
        Ok(())
    }

    pub fn seek(&self, offset: usize, limit: usize) -> Result<Vec<u8>> {
        if self.len() < offset + limit {
            return Err(IMError::NotEnoughData);
        }
        Ok(Vec::from(
            &self.buf[self.start + offset..self.start + offset + limit],
        ))
    }

    pub fn read(&mut self, offset: usize, limit: usize) -> Result<Vec<u8>> {
        if self.len() < offset + limit {
            return Err(IMError::NotEnoughData);
        }
        self.start += offset;
        let v = Vec::from(&self.buf[self.start..self.start + limit]);
        self.start += limit;
        Ok(v)
    }
}
