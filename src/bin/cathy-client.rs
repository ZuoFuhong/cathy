use cathy::IMClient;
use log::LevelFilter;

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();
    IMClient::new().run();
}
