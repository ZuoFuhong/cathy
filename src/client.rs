use crate::proto::{
    Action::CONNECTED, Action::HEARTBEAT, Action::MSG_TO_USER, ConnectedReply, MsgToUser, Package,
};
use crate::wheel_timer;
use crate::wheel_timer::system_time_unix;
use crate::Connection;
use crate::{TimerTask, WheelTimer};
use chrono::Local;
use protobuf::Message;
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;
use std::io;
use std::io::{BufRead};

/// Client链路write检测, 默认30秒, 30秒没有向链路写入任何数据时, Client会主动向Server发送心跳数据包.
const WRITER_IDLE_TIME_SECONDS: u64 = 30;
const DEFAULT_SERVER_ADDRESS: &str = "127.0.0.1:8099";

pub struct IMClient {
    connection: Connection,
    timer: WheelTimer,
    last_seq: AtomicU64,
}

impl IMClient {
    pub fn new() -> IMClient {
        let stream =
            TcpStream::connect(DEFAULT_SERVER_ADDRESS).expect("Couldn't connect to the server...");
        let connection = Connection::new(stream);
        IMClient {
            connection,
            timer: WheelTimer::new(100, 12).unwrap(),
            last_seq: AtomicU64::new(1),
        }
    }

    pub fn run(&mut self) {
        // write空闲检测
        self.init_writer_idle_timeout();
        // 开启一个线程，接收消息
        let mut connection = self.connection.clone();
        thread::spawn(move || loop {
            match connection.read_package() {
                Ok(p) => {
                    match p.get_action() {
                        CONNECTED => {
                            let msg = ConnectedReply::parse_from_bytes(p.get_content()).unwrap();
                            println!(
                                "{} 连接成功 uid = {}, session_id = {}",
                                Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                msg.get_uid(),
                                msg.get_session_id()
                            );
                        }
                        HEARTBEAT => {
                            // nothing to do
                        }
                        MSG_TO_USER => {
                            let msg = MsgToUser::parse_from_bytes(p.get_content()).unwrap();
                            println!(
                                "{} 收到用户 uid = {} 发来的消息：{}",
                                Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                msg.get_sender_uid(),
                                msg.get_content()
                            )
                        }
                    }
                }
                Err(e) => {
                    connection.set_closed();
                    println!("Subscription interrupted {}", e);
                    return;
                }
            }
        });
        thread::sleep(Duration::from_millis(10));
        // 开启终端交互, 获取用户输入
        self.start_terminal_interaction();
    }

    fn start_terminal_interaction(&mut self) {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            if !line.starts_with("send") {
                println!("Warning: 当前仅支持消息格式：send uid content");
                continue;
            }
            let items: Vec<&str> = line.split(' ').collect();
            if items.len() != 3 {
                println!("Warning: 当前仅支持消息格式：send uid content");
                continue;
            }
            let mut uid = 0;
            if let Ok(v) = items.get(1).unwrap().parse::<u64>() {
                uid = v;
            }
            if uid == 0 {
                println!("Warning: 当前仅支持消息格式：send uid content");
                continue;
            }
            let content = items.get(2).unwrap().to_string();
            self.msg_to_user(content, uid);
        }
    }

    fn init_writer_idle_timeout(&mut self) {
        let timeout_task = WriterIdleTimeoutTask::new(self.connection.clone(), self.timer.clone());
        self.timer.new_timeout(
            Box::new(timeout_task),
            Duration::from_secs(WRITER_IDLE_TIME_SECONDS),
        );
    }

    fn msg_to_user(&mut self, content: String, receiver_id: u64) {
        let seq = self.last_seq.fetch_add(1, Ordering::SeqCst);
        let mut msg_pb = MsgToUser::new();
        msg_pb.set_seq(seq);
        msg_pb.set_sender_uid(0);
        msg_pb.set_receiver_uid(receiver_id);
        msg_pb.set_message_id(0);
        msg_pb.set_content(content);
        msg_pb.set_timestamp(wheel_timer::system_time_unix());
        let package_content = msg_pb.write_to_bytes().unwrap();

        let mut package = Package::new();
        package.set_action(MSG_TO_USER);
        package.set_content(package_content);
        self.connection
            .write_package(package, Duration::from_secs(10))
            .unwrap();
    }
}

#[derive(Clone)]
struct WriterIdleTimeoutTask {
    connection: Connection,
    timer: WheelTimer,
}

impl WriterIdleTimeoutTask {
    fn new(connection: Connection, timer: WheelTimer) -> WriterIdleTimeoutTask {
        WriterIdleTimeoutTask { connection, timer }
    }
}

impl TimerTask for WriterIdleTimeoutTask {
    fn run(&mut self) {
        if self.connection.is_closed() {
            return;
        }
        let next_delay = (WRITER_IDLE_TIME_SECONDS * 1000) as i64
            - (system_time_unix() - self.connection.get_last_write_time()) as i64;
        if next_delay <= 0 {
            println!(
                "{} trigger write idle timeout check.",
                Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
            );
            // set a new timeout.
            self.timer.new_timeout(
                Box::new(self.deref().clone()),
                Duration::from_secs(WRITER_IDLE_TIME_SECONDS),
            );

            let mut package = Package::new();
            package.set_action(HEARTBEAT);
            package.set_content("PING".as_bytes().to_vec());
            self.connection
                .write_package(package, Duration::from_secs(10))
                .unwrap();
        } else {
            // set a new timeout with shorter delay.
            self.timer.new_timeout(
                Box::new(self.deref().clone()),
                Duration::from_millis(next_delay as u64),
            )
        }
    }
}
