use std::sync::Arc;
use std::io::Write;
use std::error::Error;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::fs::{self, OpenOptions};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::SendEvt;
use crate::unix_socket_handler::SocketHandler;

// open a file and append the given string to it,
// the file will be made but not the directory's
// nothing is added to the string
// it would be nice to have easy async for theses
fn append_to_file(
    the_file_path: &PathBuf,
    to_write: &str,
) -> Result<(), Box<dyn Error>> {
    if !the_file_path.exists() {
        fs::File::create(&the_file_path)
            .map_err(|err| format!("Append File Error: {}", err))?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(the_file_path)?;

    writeln!(file, "{}", to_write)?;

    Ok(())
}

/// a struct to hang methods on
pub struct Daemon {
    quiet: bool,
}

impl Daemon {
    pub fn new(quiet: bool) -> Self {
        Daemon { quiet }
    }

    pub fn default() -> Self {
        Daemon { quiet: true }
    }

    /// main run loop
    /// start the main threads and wait for input from the cli
    pub fn run(&mut self) -> Result<bool, Box<dyn Error>> {
        let main_path = Arc::new(PathBuf::from("/tmp/spellholdd_socket"));
        let mut main_socket = SocketHandler::new(&main_path);

        let log_root = PathBuf::from("/home/chris/proj/spellhold/log_files");

        let (client_accept, client_sender) =
            main_socket.get_client_handles()?;

        for next in main_socket {
            match next {
                SendEvt::Connect(val) => {
                    let log_id: &str = &val.split(' ').last().unwrap().trim();

                    let log_file = log_root.join(log_id);

                    let since_epoch = SystemTime::now()
                        .duration_since(UNIX_EPOCH)?
                        .as_secs()
                        .to_string();

                    append_to_file(
                        &log_file,
                        &format!("{} - connected\n", since_epoch),
                    )?;
                }
                SendEvt::SendString(val) => {
                    let str_split = &val.split(' ').collect::<Vec<&str>>();

                    let (log_id, content) = (str_split[0], str_split[2]);

                    let log_file = log_root.join(log_id);

                    append_to_file(&log_file, &content)?;

                    if !self.quiet {
                        println!("{}", val);
                    }

                    if client_accept.load(Ordering::Relaxed) {
                        // send the whole string to be processed by the client
                        client_sender.send(SendEvt::SendString(val)).map_err(
                            |err| format!("Error sending to client: {}", err),
                        )?;
                    }
                }
                SendEvt::Kill => {
                    if client_accept.load(Ordering::Relaxed) {
                        client_sender.send(SendEvt::Kill).map_err(|err| {
                            format!("Error killing the client: {}", err)
                        })?;

                        client_accept.store(false, Ordering::Relaxed);
                    }

                    break;
                }
                SendEvt::End | SendEvt::None => continue,
            }
        }

        Ok(false)
    }
}
