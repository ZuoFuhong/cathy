use crate::proto::Package;
use crate::wheel_timer;
use crate::IMError;
use crate::Result;
use crate::{Buffer, Codec};
use std::borrow::BorrowMut;
use std::io::Write;
use std::net::{Shutdown, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub struct Connection {
    stream: TcpStream,
    buffer: Buffer,
    closed: Arc<AtomicBool>,
    last_read_time: Arc<AtomicU64>,
    last_write_time: Arc<AtomicU64>,
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        Connection {
            stream: self.stream.try_clone().unwrap(),
            buffer: Buffer::new(),
            closed: self.closed.clone(),
            last_read_time: self.last_read_time.clone(),
            last_write_time: self.last_write_time.clone(),
        }
    }
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            buffer: Buffer::new(),
            closed: Arc::new(AtomicBool::new(false)),
            last_read_time: Arc::new(AtomicU64::new(0)),
            last_write_time: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn write_package(&mut self, p: Package, write_timeout: Duration) -> Result<()> {
        let buffer = Codec::encode(p)?;
        self.stream.set_write_timeout(Option::Some(write_timeout))?;
        self.stream.write(&buffer)?;
        self.stream.flush()?;

        self.last_write_time
            .store(wheel_timer::system_time_unix(), Ordering::SeqCst);
        Ok(())
    }

    pub fn read_package(&mut self) -> Result<Package> {
        loop {
            match Codec::decode(self.buffer.borrow_mut()) {
                Ok(p) => {
                    return Ok(p);
                }
                Err(e) => match e {
                    IMError::ContentMaxLen => {
                        return Err(e);
                    }
                    _ => {
                        self.buffer.read_from_reader(self.stream.borrow_mut())?;
                        self.last_read_time
                            .store(wheel_timer::system_time_unix(), Ordering::SeqCst);
                    }
                },
            }
        }
    }

    pub fn remote_address(&self) -> String {
        self.stream.peer_addr().unwrap().to_string()
    }

    pub fn shutdown(&mut self) {
        if self.is_closed() {
            return;
        }
        self.closed.store(true, Ordering::SeqCst);
        self.stream
            .shutdown(Shutdown::Both)
            .expect("shutdown call failed");
    }

    pub fn set_closed(&mut self) {
        self.closed.store(true, Ordering::SeqCst);
    }

    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    pub fn get_last_read_time(&self) -> u64 {
        self.last_read_time.load(Ordering::SeqCst)
    }

    pub fn get_last_write_time(&self) -> u64 {
        self.last_write_time.load(Ordering::SeqCst)
    }
}
