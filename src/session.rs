use crate::Connection;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;

#[derive(Clone)]
pub struct Session {
    session_id: String,
    uid: u64,
    connection: Connection,
}

impl Session {
    pub fn new(uid: u64, connection: Connection) -> Session {
        let uuid = Uuid::new_v4();
        Session {
            session_id: uuid.to_string(),
            uid,
            connection,
        }
    }

    pub fn get_session_id(&self) -> String {
        self.session_id.clone()
    }

    pub fn get_uid(&self) -> u64 {
        self.uid
    }

    pub fn borrow_connection(&mut self) -> &mut Connection {
        self.connection.borrow_mut()
    }
}

pub struct SessionManager {
    last_uid: AtomicU64,
    session_map: HashMap<String, Session>, // 管理会话session, key => session_id, value => session
}

impl SessionManager {
    pub fn new() -> SessionManager {
        SessionManager {
            last_uid: AtomicU64::new(1),
            session_map: HashMap::new(),
        }
    }

    pub fn new_session(&mut self, connection: Connection) -> Session {
        let uid = self.last_uid.fetch_add(1, Ordering::SeqCst);
        let session = Session::new(uid, connection);
        self.store(session.clone());
        session
    }

    fn store(&mut self, session: Session) {
        let value = session.clone();
        self.session_map.insert(session.session_id, value);
    }

    pub fn load(&self, uid: u64) -> Option<Session> {
        for (_, value) in self.session_map.clone() {
            if value.uid == uid {
                return Some(value);
            }
        }
        None
    }

    pub fn exist(&self, uid: u64) -> bool {
        for (_, value) in self.session_map.clone() {
            if value.uid == uid {
                return true;
            }
        }
        false
    }

    pub fn remove(&mut self, uid: u64) -> Option<Session> {
        match self.load(uid) {
            None => return None,
            Some(v) => {
                let ref session_id = v.session_id;
                self.session_map.remove(session_id)
            }
        }
    }

    pub fn online_users(&self) -> Vec<u64> {
        let mut uids = Vec::new();
        for (_, value) in self.session_map.clone() {
            uids.push(value.uid);
        }
        uids
    }
}
