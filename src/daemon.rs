use std::fs;
use std::io::Write;
use std::fs::OpenOptions;
use std::error::Error;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

use crate::SendEvt;
use crate::listener_socket::Listener;
use crate::client_socket::Client;

#[derive(Default)]
pub struct Daemon();

impl Daemon {
    pub fn new() -> Daemon {
        Daemon()
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let main_path = PathBuf::from("/tmp/spellholdd_socket");
        let main_socket = Listener::new(&main_path);

        let socket_path = PathBuf::from("/tmp/spellhold_client");
        let client = Client::new(&socket_path);

        for next in main_socket {
            match next {
                SendEvt::SendString(val) => {
                    let log_file = PathBuf::from("log_file");

                    if !log_file.exists() {
                        fs::File::create(log_file).unwrap();
                    }

                    let mut file = match OpenOptions::new()
                        .write(true)
                        .append(true)
                        .open("log_file")
                    {
                        Err(err) => {
                            return Err(Box::from(format!(
                                "from client {}",
                                err
                            )));
                        }
                        Ok(f) => f,
                    };

                    if let Err(err) = writeln!(file, "{}", val) {
                        eprintln!("Couldn't write to file: {}", err);
                    }

                    if client.client_accept.load(Ordering::Relaxed) {
                        client.sender.send(val).unwrap();
                    }
                }
                SendEvt::Kill => break,
                SendEvt::None => continue,
                SendEvt::Err(err) => eprintln!("Error {}", err),
            }
        }

        Ok(())
    }
}
