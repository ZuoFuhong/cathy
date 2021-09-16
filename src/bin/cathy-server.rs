use cathy::IMServer;
use log::{info, LevelFilter};

const DEFAULT_LISTENING_ADDRESS: &str = "127.0.0.1:8099";

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();
    info!("Server listen on {}", DEFAULT_LISTENING_ADDRESS);
    IMServer::new().run(DEFAULT_LISTENING_ADDRESS)
}
