use std::error::Error;
use std::path::PathBuf;
use std::{fs, thread};
use std::os::unix::net::UnixListener;
use std::sync::mpsc::{self, Receiver};
use std::io::{BufRead, BufReader};

use crate::SendEvt;

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

        for stream in listener.incoming() {
            let stream = stream.expect("no stream");
            let bf_stream = BufReader::new(stream);

            for line in bf_stream.lines() {
                let line = line.expect("bad utf8?");

                if line == "end" {
                    break;
                }

                sender
                    .send(SendEvt::SendString(line))
                    .expect("fucked sending");

            }
        }
    });

    Ok(())
}

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
