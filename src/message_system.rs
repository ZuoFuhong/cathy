use std::sync::atomic::{AtomicU64, Ordering};

pub struct MessageSystem {
    last_message_id: AtomicU64,
}

impl MessageSystem {
    pub fn new() -> MessageSystem {
        MessageSystem {
            last_message_id: AtomicU64::new(10000),
        }
    }

    pub fn next_seq(&self) -> u64 {
        self.last_message_id.fetch_add(1, Ordering::SeqCst)
    }
}
