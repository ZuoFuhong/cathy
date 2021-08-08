use crate::proto::{
    Action::CONNECTED, Action::HEARTBEAT, Action::MSG_TO_USER, ConnectedReply, MsgToUser, Package,
};
use crate::wheel_timer::system_time_unix;
use crate::SessionManager;
use crate::{Connection, WheelTimer};
use crate::{MessageSystem, TimerTask};
use chrono::Local;
use protobuf::Message;
use std::net::TcpListener;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

/// Server 链路read空闲检测, 默认60秒, 60秒没有读取到任何数据强制关闭连接.
const READER_IDLE_TIME_SECONDS: u64 = 60;

pub struct IMServer {
    session_manager: Arc<Mutex<SessionManager>>,
    message_system: Arc<Mutex<MessageSystem>>,
    timer: WheelTimer,
}

impl IMServer {
    pub fn new() -> IMServer {
        IMServer {
            session_manager: Arc::new(Mutex::new(SessionManager::new())),
            message_system: Arc::new(Mutex::new(MessageSystem::new())),
            timer: WheelTimer::new(100, 12).unwrap(),
        }
    }

    // Run the server listening on the given address
    pub fn run(&mut self, address: &str) {
        let listener = TcpListener::bind(address).unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let connection = Connection::new(stream);
                    let session = self
                        .session_manager
                        .lock()
                        .unwrap()
                        .new_session(connection.clone());

                    println!(
                        "new conn uid = {}, remote_address = {}",
                        session.get_uid(),
                        connection.remote_address()
                    );
                    // read idle detect
                    self.init_reader_idle_timeout(session.get_uid(), connection.clone());

                    let mut handler = Handler::new(
                        session.get_uid(),
                        connection,
                        self.session_manager.clone(),
                        self.message_system.clone(),
                    );
                    thread::spawn(move || handler.run());
                }
                Err(e) => {
                    panic!("Connection failed: {}", e)
                }
            }
        }
    }

    fn init_reader_idle_timeout(&mut self, uid: u64, connection: Connection) {
        let timeout_task = ReaderIdleTimeoutTask::new(
            uid,
            connection,
            self.timer.clone(),
            self.session_manager.clone(),
        );
        self.timer.new_timeout(
            Box::new(timeout_task),
            Duration::new(READER_IDLE_TIME_SECONDS, 0),
        );
    }
}

#[derive(Clone)]
struct ReaderIdleTimeoutTask {
    uid: u64,
    connection: Connection,
    timer: WheelTimer,
    session_manager: Arc<Mutex<SessionManager>>,
}

impl ReaderIdleTimeoutTask {
    fn new(
        uid: u64,
        connection: Connection,
        timer: WheelTimer,
        session_manager: Arc<Mutex<SessionManager>>,
    ) -> ReaderIdleTimeoutTask {
        ReaderIdleTimeoutTask {
            uid,
            connection,
            timer,
            session_manager,
        }
    }
}

impl TimerTask for ReaderIdleTimeoutTask {
    fn run(&mut self) {
        if self.connection.is_closed() {
            return;
        }
        let last_read_time = self.connection.get_last_read_time();
        let next_delay =
            (READER_IDLE_TIME_SECONDS * 1000) as i64 - (system_time_unix() - last_read_time) as i64;
        if next_delay <= 0 {
            println!(
                "{} trigger read idle timeout check.",
                Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
            );
            // shutdown the connection.
            self.connection.shutdown();
            // remove session
            self.session_manager.lock().unwrap().remove(self.uid);
        } else {
            // set a new timeout with shorter delay.
            self.timer.new_timeout(
                Box::new(self.deref().clone()),
                Duration::from_millis(next_delay as u64),
            );
        }
    }
}

struct Handler {
    uid: u64,
    connection: Connection,
    session_manager: Arc<Mutex<SessionManager>>,
    message_system: Arc<Mutex<MessageSystem>>,
}

impl Handler {
    fn new(
        uid: u64,
        connection: Connection,
        session_manager: Arc<Mutex<SessionManager>>,
        message_system: Arc<Mutex<MessageSystem>>,
    ) -> Handler {
        Handler {
            uid,
            connection,
            session_manager,
            message_system,
        }
    }

    fn run(&mut self) {
        self.connected_reply();
        loop {
            match self.connection.read_package() {
                Ok(p) => match p.action {
                    HEARTBEAT => {
                        println!(
                            "{} 收到 uid = {} 心跳消息：{}",
                            Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                            self.uid,
                            String::from_utf8_lossy(p.get_content())
                        );
                        let mut package = Package::new();
                        package.set_action(HEARTBEAT);
                        package.set_content("PONG".as_bytes().to_vec());
                        self.connection
                            .write_package(package, Duration::new(10, 0))
                            .unwrap();
                    }
                    MSG_TO_USER => {
                        let ret = MsgToUser::parse_from_bytes(p.get_content());
                        match ret {
                            Ok(v) => self.msg_to_user(v),
                            Err(_) => {
                                self.connection.shutdown();
                                return;
                            }
                        }
                    }
                    _ => {
                        println!(
                            "{} Unknown package action.",
                            Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        )
                    }
                },
                Err(_) => {
                    println!(
                        "{} 用户 uid = {} 离线.",
                        Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        self.uid
                    );
                    self.connection.set_closed();
                    self.session_manager.lock().unwrap().remove(self.uid);
                    return;
                }
            }
        }
    }

    fn connected_reply(&mut self) {
        let option = self.session_manager.lock().unwrap().load(self.uid);
        match option {
            Some(session) => {
                let mut reply = ConnectedReply::new();
                reply.set_uid(session.get_uid());
                reply.set_session_id(session.get_session_id());
                let content = reply.write_to_bytes().unwrap();

                let mut package = Package::new();
                package.set_action(CONNECTED);
                package.set_content(content);
                self.connection
                    .write_package(package, Duration::from_secs(10))
                    .unwrap();
            }
            None => {
                // nothing to do
            }
        }
    }

    fn msg_to_user(&self, mut mtu_pb: MsgToUser) {
        let option = self
            .session_manager
            .lock()
            .unwrap()
            .load(mtu_pb.get_receiver_uid());
        match option {
            Some(mut session) => {
                // 持久化DB，生成消息ID
                let message_id = self.message_system.lock().unwrap().next_seq();
                mtu_pb.set_message_id(message_id);
                mtu_pb.set_sender_uid(self.uid);
                let content = mtu_pb.write_to_bytes().unwrap();

                let mut package = Package::new();
                package.set_action(MSG_TO_USER);
                package.set_content(content);

                let connection = session.borrow_connection();
                let _ = connection.write_package(package, Duration::new(10, 0));
            }
            None => {
                println!("No user with uid = {} was found", mtu_pb.get_receiver_uid())
            }
        }
    }
}
