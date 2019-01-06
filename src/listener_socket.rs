use std::{fs, thread};
use std::error::Error;
use std::path::PathBuf;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixListener;
use std::sync::mpsc::{self, Receiver};

use crate::SendEvt;

pub struct Listener {
    pub receiver: Receiver<SendEvt>,
}

impl Listener {
    pub fn new(socket_path: &PathBuf) -> Listener {
        let (tx, rx) = mpsc::channel();

        let socket_path1 = socket_path.clone();

        let tx1 = tx.clone();

        thread::spawn(move || {
            if let Err(err) = stream_handler(&socket_path1, tx1) {
                eprintln!("Listener thread Error: {}", err);
            };
        });

        Listener { receiver: rx }
    }
}

impl Iterator for Listener {
    type Item = SendEvt;

    fn next(&mut self) -> Option<SendEvt> {
        match self.receiver.recv() {
            Ok(val) => Some(val),
            Err(err) => Some(SendEvt::Err(err.to_string())),
        }
    }
}

fn stream_handler(
    socket_path: &PathBuf,
    sender: mpsc::Sender<SendEvt>,
) -> Result<(), Box<dyn Error>> {
    if socket_path.exists() {
        fs::remove_file(socket_path).unwrap();
    }

    let socket_path1 = socket_path.clone();

    thread::spawn(move || {
        // get a handle of a socket
        let listener = match UnixListener::bind(&socket_path1) {
            Ok(listener) => listener,
            Err(err) => {
                let path = socket_path1.to_string_lossy();
                panic!("couldn't connect to socket_path {} {}", path, err);
            }
        };

        // TODO: make a brake point
        loop {
            let stream = match listener.accept() {
                Ok((stream, _)) => stream,
                Err(err) => {
                    panic!("thread error: {}", err);
                }
            };

            let share_stream =
                stream.try_clone().expect("cant clone stream for thread");

            let sender1 = sender.clone();

            thread::spawn(move || {
                let bf_stream = BufReader::new(share_stream);

                for line in bf_stream.lines() {
                    let line = line.expect("bad utf8?");

                    let evt = evt_dispatch(line);

                    if let SendEvt::Kill = evt {
                        sender1.send(evt).expect("fucked sending");
                        break;
                    }

                    sender1.send(evt).expect("fucked sending");
                }
            });
        }
    });

    Ok(())
}

fn evt_dispatch(line: String) -> SendEvt {
    if line.starts_with("connect") {
        SendEvt::Connect(line)
    } else if line.starts_with("kill") {
        SendEvt::Kill
    } else if line.starts_with("end") {
        SendEvt::None
    } else if !line.is_empty() {
        SendEvt::SendString(line)
    } else {
        // idk how well get here
        SendEvt::None
    }
}
