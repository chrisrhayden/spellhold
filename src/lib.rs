// extern crate spellhold;

pub mod listener_socket;
pub mod client_socket;
pub mod daemon;


#[derive(Debug)]
pub enum SendEvt {
    Kill,
    SendString(String),
    Err(String),
    None,
}

