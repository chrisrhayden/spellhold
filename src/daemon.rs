use std::fs;
use std::io::Write;
use std::fs::OpenOptions;
use std::error::Error;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

use crate::SendEvt;
use crate::listener_socket::Listener;

fn append_to_file(
    the_file_path: &PathBuf,
    to_write: &str,
) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(the_file_path)?;

    writeln!(file, "{}", to_write)?;

    Ok(())
}

#[derive(Default)]
pub struct Daemon();

impl Daemon {
    pub fn new() -> Daemon {
        Daemon()
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let main_path = PathBuf::from("/tmp/spellholdd_socket");
        let main_socket = Listener::new(&main_path);

        let log_root = PathBuf::from("/home/chris/proj/spellhold/log_files");

        for next in main_socket {
            match next {
                SendEvt::Connect(val) => {
                    let log_id: &str = &val.split(' ').last().unwrap().trim_end();

                    let log_file = log_root.join(log_id);

                    if !log_file.exists() {
                        fs::File::create(&log_file).unwrap();
                    }

                    append_to_file(&log_file, "connected")?;
                }
                SendEvt::SendString(val) => {
                    let str_split = &val.split(' ').collect::<Vec<&str>>();

                    let (log_id, content) = (str_split[0], str_split[2]);

                    let log_file = log_root.join(log_id);

                    append_to_file(&log_file, &content)?;

                    if main_socket.client_accept.load(Ordering::Relaxed) {
                        // send the whole string to be processed by the client
                        main_socket.client_sender.send(val).unwrap();
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
