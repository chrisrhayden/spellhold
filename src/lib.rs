pub mod listener_socket;
pub mod client_socket;
pub mod daemon;

#[derive(Debug)]
pub enum SendEvt {
    Kill,
    None,
    Err(String),
    Connect(String),
    SendString(String),
}
