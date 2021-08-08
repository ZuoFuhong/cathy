use cathy::IMServer;

const DEFAULT_LISTENING_ADDRESS: &str = "127.0.0.1:8099";

fn main() {
    println!("Server start listen as 8099");
    IMServer::new().run(DEFAULT_LISTENING_ADDRESS)
}
