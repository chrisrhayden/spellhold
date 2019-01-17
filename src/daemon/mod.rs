pub mod main_loop;
pub mod unix_socket_handler;

#[derive(Debug, Clone)]
pub enum SendEvt {
    End,
    Kill,
    None,
    Restart,
    Connect(String),
    SendString(String),
}

impl SendEvt {
    pub fn new(line: String) -> SendEvt {
        SendEvt::evt_dispatch(line)
    }

    fn evt_dispatch(line: String) -> SendEvt {
        if line.starts_with("kill") {
            SendEvt::Kill
        } else if line.starts_with("end") {
            SendEvt::End
        } else if line.starts_with("restart") {
            SendEvt::Restart
        } else if !line.is_empty() {
            SendEvt::SendString(line)
        } else {
            // idk how well get here
            SendEvt::None
        }
    }
}
